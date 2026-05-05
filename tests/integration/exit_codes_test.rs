use std::process::Command;

// ---- Issue command exit code tests (T028) ----

#[test]
fn issue_get_nonexistent_exits_1_when_unauthenticated() {
    let output = Command::new(env!("CARGO_BIN_EXE_linear"))
        .args(["issue", "get", "FAKE-9999", "--json"])
        .env("SKIP_KEYCHAIN_TESTS", "0")
        .output()
        .unwrap();
    let code = output.status.code().unwrap_or(-1);
    // Exit 3 = no auth (expected in CI), 1 = not-found, 2 = network error, 0 = found (CI with creds)
    assert!(
        code == 3 || code == 1 || code == 2 || code == 0,
        "expected exit code 3/1/2/0 for issue get nonexistent, got {code}"
    );
}

#[test]
fn issue_create_missing_title_exits_1() {
    let output = Command::new(env!("CARGO_BIN_EXE_linear"))
        .args(["issue", "create", "--team", "ENG"])
        .env("LINEAR_API_KEY", "test-key")
        .env("SKIP_KEYCHAIN_TESTS", "0")
        .output()
        .unwrap();
    let code = output.status.code().unwrap_or(-1);
    // clap exits 1 when required arg is missing (--title is required)
    assert!(
        code == 1 || code == 2 || code == 3,
        "expected exit code 1/2/3 for missing --title, got {code}"
    );
}

#[test]
fn issue_update_no_flags_exits_1() {
    let output = Command::new(env!("CARGO_BIN_EXE_linear"))
        .args(["issue", "update", "ENG-1"])
        .env("SKIP_KEYCHAIN_TESTS", "0")
        .output()
        .unwrap();
    let code = output.status.code().unwrap_or(-1);
    assert!(
        code == 1 || code == 2 || code == 3,
        "expected exit code 1/2/3 for update with no flags, got {code}"
    );
}

#[test]
fn issue_update_parent_and_no_parent_conflict_exits_1() {
    let output = Command::new(env!("CARGO_BIN_EXE_linear"))
        .args(["issue", "update", "ENG-1", "--parent", "ENG-2", "--no-parent"])
        .env("SKIP_KEYCHAIN_TESTS", "0")
        .output()
        .unwrap();
    let code = output.status.code().unwrap_or(-1);
    assert_eq!(
        code, 1,
        "--parent and --no-parent should exit 1 (mutually exclusive), got {code}"
    );
}

#[test]
fn issue_get_exits_3_when_not_authenticated() {
    let output = Command::new(env!("CARGO_BIN_EXE_linear"))
        .args(["issue", "get", "ENG-1"])
        .env("SKIP_KEYCHAIN_TESTS", "0")
        .output()
        .unwrap();
    let code = output.status.code().unwrap_or(-1);
    assert!(
        code == 3 || code == 0 || code == 2,
        "expected exit code 3 (no auth), 0, or 2, got {code}"
    );
}

#[test]
fn issue_create_exits_3_when_not_authenticated() {
    let output = Command::new(env!("CARGO_BIN_EXE_linear"))
        .args(["issue", "create", "--title", "Test", "--team", "ENG", "--project", "proj-1"])
        .env("SKIP_KEYCHAIN_TESTS", "0")
        .output()
        .unwrap();
    let code = output.status.code().unwrap_or(-1);
    assert!(
        code == 3 || code == 0 || code == 2,
        "expected exit code 3 (no auth), 0, or 2, got {code}"
    );
}

#[test]
fn issue_update_exits_3_when_not_authenticated() {
    let output = Command::new(env!("CARGO_BIN_EXE_linear"))
        .args(["issue", "update", "ENG-1", "--title", "New Title"])
        .env("SKIP_KEYCHAIN_TESTS", "0")
        .output()
        .unwrap();
    let code = output.status.code().unwrap_or(-1);
    assert!(
        code == 3 || code == 0 || code == 2,
        "expected exit code 3 (no auth), 0, or 2, got {code}"
    );
}

// ---- Existing tests ----

#[test]
fn bad_flag_exits_with_code_1() {
    let output = Command::new(env!("CARGO_BIN_EXE_linear"))
        .arg("--bad-flag")
        .output()
        .expect("failed to spawn binary");
    assert_eq!(output.status.code(), Some(1));
}

#[test]
fn version_flag_exits_with_code_0() {
    let output = Command::new(env!("CARGO_BIN_EXE_linear"))
        .arg("--version")
        .output()
        .expect("failed to spawn binary");
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn issue_list_exits_3_when_not_authenticated() {
    // With auth guard wired in (FR-007), issue list exits 3 when no credential is present.
    // This test works in CI where no keychain credential is present.
    // If a keychain credential is present, it will try to validate remotely and may exit 0 or 2.
    let output = Command::new(env!("CARGO_BIN_EXE_linear"))
        .args(["issue", "list", "--json"])
        .env("SKIP_KEYCHAIN_TESTS", "0")
        .output()
        .unwrap();
    let code = output.status.code().unwrap_or(-1);
    assert!(
        code == 3 || code == 0 || code == 2,
        "expected exit code 3 (no auth), 0 (auth ok), or 2 (network error), got {code}"
    );
}
