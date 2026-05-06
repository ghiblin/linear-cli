use std::process::Command;

fn linear() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_linear"));
    cmd.env("SKIP_KEYCHAIN_TESTS", "0");
    cmd
}

fn accepted(code: i32) -> bool {
    code != 2
}

#[test]
fn team_list_accepts_json_flag() {
    let code = linear()
        .args(["team", "list", "--json"])
        .output()
        .unwrap()
        .status
        .code()
        .unwrap_or(-1);
    assert!(
        accepted(code),
        "team list --json rejected by clap (exit {code})"
    );
}

#[test]
fn team_list_accepts_output_json_flag() {
    let code = linear()
        .args(["team", "list", "--output", "json"])
        .output()
        .unwrap()
        .status
        .code()
        .unwrap_or(-1);
    assert!(
        accepted(code),
        "team list --output json rejected by clap (exit {code})"
    );
}
