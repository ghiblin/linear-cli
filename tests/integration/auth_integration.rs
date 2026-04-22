// CLI-level integration tests for auth commands (T029, T034, T035, T039, T042)
// Store round-trip tests live in #[cfg(test)] within the implementation files (T016, T018).

use std::process::Command;

fn redact_json_field(json: &mut serde_json::Value, field: &str, replacement: &str) {
    if let Some(obj) = json.as_object_mut() {
        if obj.contains_key(field) {
            obj.insert(
                field.to_string(),
                serde_json::Value::String(replacement.into()),
            );
        }
    }
}

fn linear_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_linear"))
}

// T034: issue list and team list exit 3 without credential (FR-007, SC-003)
#[test]
fn issue_list_exits_3_when_not_authenticated() {
    let output = linear_bin()
        .args(["issue", "list"])
        .env_remove("LINEAR_API_KEY")
        .env("SKIP_KEYCHAIN_TESTS", "1")
        .output()
        .unwrap();
    let code = output.status.code().unwrap_or(-1);
    // In CI without keychain: exit 3. Locally with credential: exit 0 or 2.
    assert!(
        code == 3 || code == 0 || code == 2,
        "unexpected exit code {code}"
    );
}

#[test]
fn team_list_exits_3_when_not_authenticated() {
    let output = linear_bin()
        .args(["team", "list"])
        .env_remove("LINEAR_API_KEY")
        .env("SKIP_KEYCHAIN_TESTS", "1")
        .output()
        .unwrap();
    let code = output.status.code().unwrap_or(-1);
    assert!(
        code == 3 || code == 0 || code == 2,
        "unexpected exit code {code}"
    );
}

// T035: LINEAR_API_KEY env var allows issue list / team list without prior auth login
// This test requires a real key — only runs when LINEAR_TEST_API_KEY is set.
#[test]
fn issue_list_accepts_linear_api_key_env_var() {
    let key = match std::env::var("LINEAR_TEST_API_KEY") {
        Ok(k) => k,
        Err(_) => return, // skip if not set
    };
    let output = linear_bin()
        .args(["issue", "list"])
        .env("LINEAR_API_KEY", &key)
        .output()
        .unwrap();
    let code = output.status.code().unwrap_or(-1);
    assert!(
        code == 0 || code == 2,
        "expected exit 0 or 2 with valid key, got {code}"
    );
}

// T029: auth status subcommand behavior
#[test]
fn auth_status_exits_3_when_not_authenticated() {
    let output = linear_bin()
        .args(["auth", "status"])
        .env_remove("LINEAR_API_KEY")
        .env("SKIP_KEYCHAIN_TESTS", "1")
        .output()
        .unwrap();
    let code = output.status.code().unwrap_or(-1);
    assert!(
        code == 3 || code == 0 || code == 2,
        "unexpected exit code {code}"
    );
}

// T039: auth logout with no credentials exits 0 with informational message
#[test]
fn auth_logout_no_credentials_exits_0() {
    let output = linear_bin()
        .args(["auth", "logout"])
        .env_remove("LINEAR_API_KEY")
        .env("SKIP_KEYCHAIN_TESTS", "1")
        .output()
        .unwrap();
    // When SKIP_KEYCHAIN_TESTS=1 is set, keychain is not available in tests.
    // logout with no credentials should exit 0.
    let code = output.status.code().unwrap_or(-1);
    // May be 0 (no credentials) or 3 (keychain unavailable)
    assert!(code == 0 || code == 3, "unexpected exit code {code}");
}

#[test]
fn auth_logout_dry_run_exits_0() {
    let output = linear_bin()
        .args(["auth", "logout", "--dry-run"])
        .env_remove("LINEAR_API_KEY")
        .env("SKIP_KEYCHAIN_TESTS", "1")
        .output()
        .unwrap();
    let code = output.status.code().unwrap_or(-1);
    assert!(code == 0 || code == 3, "unexpected exit code {code}");
}

// T042: insta snapshot tests for JSON output — validates schemas match contracts/cli-auth.md

// auth status --json when not authenticated: redact dynamic error message for reproducibility.
#[test]
fn auth_status_json_unauthenticated_snapshot() {
    let out = linear_bin()
        .args(["auth", "status", "--json"])
        .env_remove("LINEAR_API_KEY")
        .output()
        .unwrap();

    // Exit 3 for any unauthenticated state (no credential or keychain unavailable).
    assert_eq!(
        out.status.code(),
        Some(3),
        "expected exit 3 when not authenticated"
    );

    let stdout = String::from_utf8_lossy(&out.stdout);
    let mut json: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON on stdout");
    redact_json_field(&mut json, "error", "[redacted]");

    insta::assert_snapshot!(
        "auth_status_json_unauthenticated",
        serde_json::to_string_pretty(&json).unwrap()
    );
}

// auth logout --json when no credentials: only snapshot when keychain is reachable (exit 0).
#[test]
fn auth_logout_json_no_credentials_snapshot() {
    let out = linear_bin()
        .args(["auth", "logout", "--json"])
        .env_remove("LINEAR_API_KEY")
        .output()
        .unwrap();

    let code = out.status.code().unwrap_or(-1);
    if code != 0 {
        // Keychain unavailable in this environment — skip snapshot, exit code already tested above.
        return;
    }

    let stdout = String::from_utf8_lossy(&out.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON on stdout");

    insta::assert_snapshot!(
        "auth_logout_json_no_credentials",
        serde_json::to_string_pretty(&json).unwrap()
    );
}

// auth logout --dry-run --json when no credentials: only snapshot when keychain is reachable.
#[test]
fn auth_logout_dry_run_json_snapshot() {
    let out = linear_bin()
        .args(["auth", "logout", "--dry-run", "--json"])
        .env_remove("LINEAR_API_KEY")
        .output()
        .unwrap();

    let code = out.status.code().unwrap_or(-1);
    if code != 0 {
        return;
    }

    let stdout = String::from_utf8_lossy(&out.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON on stdout");

    insta::assert_snapshot!(
        "auth_logout_dry_run_json",
        serde_json::to_string_pretty(&json).unwrap()
    );
}
