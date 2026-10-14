use std::{
    ffi::OsString,
    process::{Command, Stdio},
    sync::atomic::{AtomicBool, Ordering},
    thread,
    time::{Duration, Instant},
};

use anyhow::{Context, Result, bail, ensure};
use serde_json::Value;

use crate::model::{Snapshot, Target};

const TIMEOUT: Duration = Duration::from_secs(5);
const OUTPUT_LIMIT: u64 = 32 * 1024 * 1024;

pub struct Herdr {
    pub binary: OsString,
}

impl Default for Herdr {
    fn default() -> Self {
        Self {
            binary: std::env::var_os("HERDR_BIN_PATH").unwrap_or_else(|| "herdr".into()),
        }
    }
}

pub fn decode(bytes: &[u8]) -> Result<Value> {
    let response: Value = serde_json::from_slice(bytes).context("invalid Herdr JSON response")?;
    if let Some(error) = response.get("error") {
        bail!(
            "Herdr {}: {}",
            error["code"].as_str().unwrap_or("error"),
            error["message"].as_str().unwrap_or("request failed")
        );
    }
    response
        .get("result")
        .filter(|result| result.is_object())
        .cloned()
        .context("Herdr response has no object result")
}

pub fn parse_snapshot(bytes: &[u8]) -> Result<Snapshot> {
    let result = decode(bytes)?;
    let mut snapshot: Snapshot = serde_json::from_value(
        result
            .get("snapshot")
            .cloned()
            .context("Herdr response has no snapshot")?,
    )
    .context("invalid Herdr snapshot")?;
    snapshot.validate()?;
    Ok(snapshot)
}

pub fn focus_args(target: &Target) -> Vec<&str> {
    match target {
        Target::Workspace(id) => vec!["workspace", "focus", id],
        Target::Tab(id) => vec!["tab", "focus", id],
    }
}

impl Herdr {
    pub fn call(&self, args: &[&str]) -> Result<Vec<u8>> {
        self.call_cancellable(args, &AtomicBool::new(false))
    }

    fn call_cancellable(&self, args: &[&str], cancelled: &AtomicBool) -> Result<Vec<u8>> {
        use std::io::Read;
        let mut child = Command::new(&self.binary)
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("start Herdr CLI (check HERDR_BIN_PATH)")?;
        let stdout = child.stdout.take().context("missing CLI stdout")?;
        let stderr = child.stderr.take().context("missing CLI stderr")?;
        // Drain both pipes concurrently so a large snapshot cannot block the child.
        let out = thread::spawn(move || {
            let mut bytes = Vec::new();
            stdout
                .take(OUTPUT_LIMIT + 1)
                .read_to_end(&mut bytes)
                .map(|_| bytes)
        });
        let err = thread::spawn(move || {
            let mut bytes = Vec::new();
            stderr
                .take(OUTPUT_LIMIT + 1)
                .read_to_end(&mut bytes)
                .map(|_| bytes)
        });
        let deadline = Instant::now() + TIMEOUT;
        let status = loop {
            if cancelled.load(Ordering::Relaxed) {
                let _ = child.kill();
                let _ = child.wait();
                break Err(anyhow::anyhow!("Herdr request cancelled"));
            }
            match child.try_wait() {
                Ok(Some(status)) => break Ok(status),
                Ok(None) if Instant::now() < deadline => thread::sleep(Duration::from_millis(5)),
                Ok(None) => {
                    let _ = child.kill();
                    let _ = child.wait();
                    break Err(anyhow::anyhow!("Herdr request timed out"));
                }
                Err(e) => {
                    let _ = child.kill();
                    let _ = child.wait();
                    break Err(e.into());
                }
            }
        };
        let stdout = out
            .join()
            .map_err(|_| anyhow::anyhow!("CLI stdout reader failed"))??;
        let stderr = err
            .join()
            .map_err(|_| anyhow::anyhow!("CLI stderr reader failed"))??;
        let status = status?;
        if !status.success() {
            // API errors sometimes come on stdout even when the CLI exits nonzero.
            if !stdout.is_empty() {
                decode(&stdout)?;
            }
            bail!(
                "Herdr request failed: {}",
                String::from_utf8_lossy(&stderr).trim()
            );
        }
        if stdout.len() as u64 > OUTPUT_LIMIT {
            bail!("Herdr response exceeds 32 MiB");
        }
        Ok(stdout)
    }

    pub fn snapshot(&self) -> Result<Snapshot> {
        parse_snapshot(&self.call(&["api", "snapshot"])?)
    }

    pub fn snapshot_cancellable(&self, cancelled: &AtomicBool) -> Result<Snapshot> {
        parse_snapshot(&self.call_cancellable(&["api", "snapshot"], cancelled)?)
    }

    pub fn open(&self) -> Result<()> {
        decode(&self.call(&[
            "plugin",
            "pane",
            "open",
            "--plugin",
            "herdr-picker",
            "--entrypoint",
            "picker",
        ])?)?;
        Ok(())
    }

    pub fn focus(&self, target: &Target) -> Result<()> {
        let result = decode(&self.call(&focus_args(target))?)?;
        let (kind, record, field, expected) = match target {
            Target::Workspace(id) => ("workspace_info", "workspace", "workspace_id", id),
            Target::Tab(id) => ("tab_info", "tab", "tab_id", id),
        };
        ensure!(
            result["type"].as_str() == Some(kind),
            "invalid Herdr focus response type"
        );
        ensure!(
            result[record][field].as_str() == Some(expected.as_str()),
            "Herdr focus response has an unexpected destination ID"
        );
        Ok(())
    }

    pub fn notify(&self, message: &str) -> Result<()> {
        decode(&self.call(&["notification", "show", "Herdr Picker", "--body", message])?)?;
        Ok(())
    }
}
