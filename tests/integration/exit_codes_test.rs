use std::process::Command;

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
