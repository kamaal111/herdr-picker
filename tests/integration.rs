use herdr_picker::{
    api::{Herdr, decode, focus_args, parse_snapshot},
    config::{Config, SearchMode},
    model::Target,
};
use serde_json::{Value, json};

fn fixture() -> Value {
    serde_json::from_slice(include_bytes!("fixtures/snapshot.json")).unwrap()
}
fn parse(value: &Value) -> anyhow::Result<herdr_picker::model::Snapshot> {
    parse_snapshot(&serde_json::to_vec(value).unwrap())
}

#[test]
fn parses_snapshot_and_ignores_unrelated_fields() {
    let snapshot = parse(&fixture()).unwrap();
    assert_eq!(snapshot.workspaces.len(), 3);
    assert_eq!(snapshot.tabs.len(), 6);
    assert_eq!(snapshot.workspaces[0].workspace_id, "w1");
    assert_eq!(snapshot.tabs[0].tab_id, "w1:t4");
    assert_eq!(snapshot.focused_workspace_id.as_deref(), Some("w2"));
    assert_eq!(snapshot.focused_tab_id.as_deref(), Some("w2:t7"));
}
#[test]
fn invalid_json_is_reported() {
    assert!(parse_snapshot(b"not JSON").is_err());
}
#[test]
fn missing_result_is_reported() {
    assert!(decode(b"{}").is_err());
}
#[test]
fn null_result_is_rejected() {
    assert!(decode(b"{\"result\":null}").is_err());
}
#[test]
fn missing_snapshot_is_reported() {
    assert!(parse(&json!({"result":{}})).is_err());
}
#[test]
fn missing_workspace_id_is_rejected() {
    let mut fixture = fixture();
    fixture["result"]["snapshot"]["workspaces"][0]
        .as_object_mut()
        .unwrap()
        .remove("workspace_id");
    assert!(parse(&fixture).is_err());
}
#[test]
fn blank_workspace_id_is_rejected() {
    let mut fixture = fixture();
    fixture["result"]["snapshot"]["workspaces"][0]["workspace_id"] = json!("");
    assert!(parse(&fixture).is_err());
}
#[test]
fn duplicate_workspace_id_is_rejected() {
    let mut fixture = fixture();
    fixture["result"]["snapshot"]["workspaces"][1]["workspace_id"] = json!("w1");
    assert!(parse(&fixture).is_err());
}
#[test]
fn orphaned_tab_is_rejected() {
    let mut fixture = fixture();
    fixture["result"]["snapshot"]["tabs"][0]["workspace_id"] = json!("unknown");
    assert!(parse(&fixture).is_err());
}
#[test]
fn missing_tab_id_is_rejected() {
    let mut fixture = fixture();
    fixture["result"]["snapshot"]["tabs"][0]
        .as_object_mut()
        .unwrap()
        .remove("tab_id");
    assert!(parse(&fixture).is_err());
}
#[test]
fn invalid_active_tab_is_rejected() {
    let mut fixture = fixture();
    fixture["result"]["snapshot"]["workspaces"][0]["active_tab_id"] = json!("w2:t7");
    assert!(parse(&fixture).is_err());
}
#[test]
fn tab_order_uses_ui_number_instead_of_id() {
    let mut fixture = fixture();
    fixture["result"]["snapshot"]["tabs"]
        .as_array_mut()
        .unwrap()
        .reverse();
    let snapshot = parse(&fixture).unwrap();
    let ids: Vec<_> = snapshot
        .tabs
        .iter()
        .filter(|t| t.workspace_id == "w1")
        .map(|t| t.tab_id.as_str())
        .collect();
    assert_eq!(ids, vec!["w1:t4", "w1:t8", "w1:t9"]);
}
#[test]
fn workspace_focus_uses_id() {
    assert_eq!(
        focus_args(&Target::Workspace("w9".into())),
        vec!["workspace", "focus", "w9"]
    );
}
#[test]
fn tab_focus_is_a_single_parent_aware_request() {
    assert_eq!(
        focus_args(&Target::Tab("w9:t8".into())),
        vec!["tab", "focus", "w9:t8"]
    );
}
#[test]
fn stale_workspace_failure_is_clear() {
    let error =
        decode(br#"{"error":{"code":"workspace_not_found","message":"workspace w9 not found"}}"#)
            .unwrap_err();
    assert_eq!(
        error.to_string(),
        "Herdr workspace_not_found: workspace w9 not found"
    );
}
#[test]
fn stale_tab_failure_is_clear() {
    let error =
        decode(br#"{"error":{"code":"tab_not_found","message":"tab closed"}}"#).unwrap_err();
    assert_eq!(error.to_string(), "Herdr tab_not_found: tab closed");
}
#[test]
fn default_search_mode_is_hierarchical() {
    assert_eq!(
        Config::parse("").unwrap().search_mode,
        SearchMode::Hierarchical
    );
}
#[test]
fn flat_configuration_parses() {
    assert_eq!(
        Config::parse("search_mode = 'flat'").unwrap().search_mode,
        SearchMode::Flat
    );
}
#[test]
fn invalid_search_mode_is_reported() {
    assert!(Config::parse("search_mode = 'tree'").is_err());
}
#[test]
fn typo_in_configuration_is_reported() {
    assert!(Config::parse("search_mod = 'flat'").is_err());
}
#[test]
fn unavailable_cli_is_reported() {
    assert!(
        Herdr {
            binary: "/nonexistent/herdr-picker-test-cli".into()
        }
        .snapshot()
        .unwrap_err()
        .to_string()
        .contains("start Herdr CLI")
    );
}

#[cfg(unix)]
mod cli {
    use super::*;
    use std::{
        fs,
        os::unix::fs::PermissionsExt,
        path::PathBuf,
        sync::atomic::{AtomicUsize, Ordering},
    };
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    struct Fake {
        path: PathBuf,
    }
    impl Fake {
        fn new(script: &str) -> Self {
            let path = std::env::temp_dir().join(format!(
                "herdr-picker-test-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&path).unwrap();
            fs::write(path.join("herdr"), format!("#!/bin/sh\n{script}\n")).unwrap();
            fs::set_permissions(path.join("herdr"), fs::Permissions::from_mode(0o700)).unwrap();
            Self { path }
        }
        fn api(&self) -> Herdr {
            Herdr {
                binary: self.path.join("herdr").into(),
            }
        }
    }
    impl Drop for Fake {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    #[test]
    fn runs_real_cli_transport_and_parses_snapshot() {
        let fake = Fake::new(&format!(
            "test \"$1 $2\" = 'api snapshot' || exit 2\ncat <<'JSON'\n{}\nJSON",
            include_str!("fixtures/snapshot.json")
        ));
        assert_eq!(fake.api().snapshot().unwrap().tabs.len(), 6);
    }
    #[test]
    fn transport_reports_nonzero_api_errors_for_stale_tabs() {
        let fake = Fake::new(
            "echo '{\"error\":{\"code\":\"tab_not_found\",\"message\":\"tab closed\"}}'\nexit 1",
        );
        assert_eq!(
            fake.api()
                .focus(&Target::Tab("w1:t8".into()))
                .unwrap_err()
                .to_string(),
            "Herdr tab_not_found: tab closed"
        );
    }
    #[test]
    fn popup_open_reports_modal_busy() {
        let fake = Fake::new(
            "echo '{\"error\":{\"code\":\"ui_busy\",\"message\":\"another modal is active\"}}'",
        );
        assert!(
            fake.api()
                .open()
                .unwrap_err()
                .to_string()
                .contains("ui_busy")
        );
    }
    #[test]
    fn focus_arguments_are_passed_without_shell_interpretation() {
        let fake = Fake::new(
            "test \"$1\" = tab && test \"$2\" = focus && test \"$3\" = 'w1:t8;echo unsafe' || exit 2\necho '{\"result\":{\"type\":\"tab_info\",\"tab\":{\"tab_id\":\"w1:t8;echo unsafe\"}}}'",
        );
        assert!(
            fake.api()
                .focus(&Target::Tab("w1:t8;echo unsafe".into()))
                .is_ok()
        );
    }
    #[test]
    fn focus_rejects_a_success_response_for_a_different_id() {
        let fake =
            Fake::new("echo '{\"result\":{\"type\":\"tab_info\",\"tab\":{\"tab_id\":\"w2:t1\"}}}'");
        assert_eq!(
            fake.api()
                .focus(&Target::Tab("w1:t8".into()))
                .unwrap_err()
                .to_string(),
            "Herdr focus response has an unexpected destination ID"
        );
    }
    #[test]
    fn cli_timeout_is_bounded() {
        let fake = Fake::new("exec sleep 30");
        assert_eq!(
            fake.api().snapshot().unwrap_err().to_string(),
            "Herdr request timed out"
        );
    }
}

#[test]
fn distributable_manifest_declares_action_and_popup_entrypoints() {
    let manifest: toml::Value = toml::from_str(include_str!("../herdr-plugin.toml")).unwrap();
    assert_eq!(manifest["id"].as_str(), Some("herdr-picker"));
    assert_eq!(manifest["panes"][0]["placement"].as_str(), Some("popup"));
    assert_eq!(manifest["actions"][0]["command"][1].as_str(), Some("open"));
    assert_eq!(manifest["panes"][0]["command"][1].as_str(), Some("pick"));
    assert_eq!(
        manifest["build"][0]["command"].as_array().unwrap(),
        &vec![
            toml::Value::from("cargo"),
            toml::Value::from("build"),
            toml::Value::from("--release"),
            toml::Value::from("--locked")
        ]
    );
}
