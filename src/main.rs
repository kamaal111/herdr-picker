use std::{env, path::PathBuf};

use anyhow::{Context, Result, bail};
use herdr_picker::{api::Herdr, config::Config, ui};

fn run(api: &Herdr) -> Result<()> {
    match env::args().nth(1).as_deref() {
        Some("open") => api.open(),
        Some("check") => {
            let snapshot = api.snapshot()?;
            println!(
                "Herdr snapshot validated: {} workspaces, {} tabs",
                snapshot.workspaces.len(),
                snapshot.tabs.len()
            );
            Ok(())
        }
        Some("pick") => {
            let directory = PathBuf::from(
                env::var_os("HERDR_PLUGIN_CONFIG_DIR")
                    .context("launch pick through the Herdr plugin popup")?,
            );
            let config = Config::load(&directory)?;
            if let Some((snapshot, target)) = ui::run(api, config.search_mode)? {
                // One focus request validates the target before changing context. Herdr's
                // tab.focus also focuses its parent; no partially completed two-step switch.
                if let Err(error) = api.focus(&target) {
                    let original = snapshot
                        .focused_tab_id
                        .clone()
                        .map(herdr_picker::model::Target::Tab)
                        .or_else(|| {
                            snapshot
                                .focused_workspace_id
                                .clone()
                                .map(herdr_picker::model::Target::Workspace)
                        });
                    if let Some(original) = original
                        && let Err(restore_error) = api.focus(&original)
                    {
                        bail!("{error:#}; restoring original focus also failed: {restore_error:#}");
                    }
                    return Err(error);
                }
            }
            // The popup closes on process exit, leaving the focused destination intact.
            Ok(())
        }
        Some("--version") | Some("-V") => {
            println!("herdr-picker {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        Some("--help") | Some("-h") | None => {
            println!(
                "herdr-picker {}\n\nopen    Launch the installed Herdr popup\npick    Popup entrypoint (launched by Herdr)\ncheck   Validate a live snapshot without changing focus",
                env!("CARGO_PKG_VERSION")
            );
            Ok(())
        }
        _ => bail!("unknown command; use --help"),
    }
}

fn main() {
    let api = Herdr::default();
    if let Err(error) = run(&api) {
        let message = format!("{error:#}");
        eprintln!("herdr-picker: {message}");
        if let Err(notification_error) = api.notify(&message) {
            eprintln!("herdr-picker: could not show error notification: {notification_error:#}");
        }
        std::process::exit(1);
    }
}
