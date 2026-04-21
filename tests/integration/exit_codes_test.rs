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
fn issue_list_json_snapshot() {
    let output = Command::new(env!("CARGO_BIN_EXE_linear"))
        .args(["issue", "list", "--json"])
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    insta::assert_snapshot!(stdout.trim());
}
