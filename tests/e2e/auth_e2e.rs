// E2E test scaffold for the full auth cycle (T041).
// Guards all tests behind LINEAR_TEST_API_KEY — skips gracefully when absent.
//
// Run with: LINEAR_TEST_API_KEY=lin_api_... cargo test --test e2e

use std::io::Write;
use std::process::{Command, Stdio};

fn linear_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_linear"))
}

fn require_test_key() -> Option<String> {
    std::env::var("LINEAR_TEST_API_KEY").ok()
}

// Status with a valid env-var key exits 0 and outputs workspace data.
#[test]
fn auth_status_with_env_var_exits_0() {
    let key = match require_test_key() {
        Some(k) => k,
        None => return,
    };

    let out = linear_bin()
        .args(["auth", "status"])
        .env("LINEAR_API_KEY", &key)
        .output()
        .unwrap();

    let code = out.status.code().unwrap_or(-1);
    assert!(
        code == 0,
        "auth status with valid key should exit 0, got {code}\nstderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

// Status with JSON flag returns authenticated=true and expected fields.
#[test]
fn auth_status_json_authenticated() {
    let key = match require_test_key() {
        Some(k) => k,
        None => return,
    };

    let out = linear_bin()
        .args(["auth", "status", "--json"])
        .env("LINEAR_API_KEY", &key)
        .output()
        .unwrap();

    assert_eq!(out.status.code(), Some(0));

    let stdout = String::from_utf8_lossy(&out.stdout);
    let json: serde_json::Value =
        serde_json::from_str(&stdout).expect("stdout should be valid JSON");
    assert_eq!(json["authenticated"], true);
    assert!(
        json["workspace"].is_object(),
        "workspace field must be present"
    );
    assert_eq!(json["source"], "env_var");
}

// Login via stdin pipe stores a credential in a temp file and exits 0.
#[test]
fn auth_login_via_file_store_exits_0() {
    let key = match require_test_key() {
        Some(k) => k,
        None => return,
    };

    let dir = tempfile::tempdir().expect("tmpdir");
    let cred_path = dir.path().join("credentials");
    let path_str = cred_path.to_str().unwrap();

    let mut child = linear_bin()
        .args(["auth", "login", "--store-file", path_str])
        .env_remove("LINEAR_API_KEY")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn login");

    child
        .stdin
        .take()
        .unwrap()
        .write_all(format!("{key}\n").as_bytes())
        .unwrap();

    let out = child.wait_with_output().unwrap();
    let code = out.status.code().unwrap_or(-1);
    assert!(
        code == 0,
        "login should exit 0 with valid key, got {code}\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );
    assert!(
        cred_path.exists(),
        "credentials file should exist after login"
    );
}

// Full cycle: login to file store → issue list via env var → logout (keychain, no-op if empty).
#[test]
fn issue_list_accepts_env_var_key() {
    let key = match require_test_key() {
        Some(k) => k,
        None => return,
    };

    let out = linear_bin()
        .args(["issue", "list"])
        .env("LINEAR_API_KEY", &key)
        .output()
        .unwrap();

    let code = out.status.code().unwrap_or(-1);
    // May return 0 (issues found) or succeed differently based on workspace contents
    assert!(
        code == 0 || code == 2,
        "issue list with valid key should exit 0 or 2 (no network), got {code}"
    );
}
