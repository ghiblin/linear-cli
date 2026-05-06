use std::process::Command;

fn linear() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_linear"));
    cmd.env("SKIP_KEYCHAIN_TESTS", "0");
    cmd
}

/// Clap exits 2 for unrecognised flags. All other codes mean the flag was accepted.
fn accepted(code: i32) -> bool {
    code != 2
}

#[test]
fn issue_list_accepts_json_flag() {
    let code = linear()
        .args(["issue", "list", "--json"])
        .output()
        .unwrap()
        .status
        .code()
        .unwrap_or(-1);
    assert!(
        accepted(code),
        "issue list --json rejected by clap (exit {code})"
    );
}

#[test]
fn issue_get_accepts_json_flag() {
    let code = linear()
        .args(["issue", "get", "ENG-1", "--json"])
        .output()
        .unwrap()
        .status
        .code()
        .unwrap_or(-1);
    assert!(
        accepted(code),
        "issue get --json rejected by clap (exit {code})"
    );
}

#[test]
fn issue_create_accepts_json_flag() {
    let code = linear()
        .args(["issue", "create", "--title", "t", "--team", "ENG", "--json"])
        .output()
        .unwrap()
        .status
        .code()
        .unwrap_or(-1);
    assert!(
        accepted(code),
        "issue create --json rejected by clap (exit {code})"
    );
}

#[test]
fn issue_update_accepts_json_flag() {
    let code = linear()
        .args(["issue", "update", "ENG-1", "--title", "t", "--json"])
        .output()
        .unwrap()
        .status
        .code()
        .unwrap_or(-1);
    assert!(
        accepted(code),
        "issue update --json rejected by clap (exit {code})"
    );
}

// T018: --json and --output json are interchangeable (both accepted, same exit behaviour)
#[test]
fn issue_list_json_flag_and_output_json_behave_identically() {
    let code_json = linear()
        .args(["issue", "list", "--json"])
        .output()
        .unwrap()
        .status
        .code()
        .unwrap_or(-1);
    let code_output = linear()
        .args(["issue", "list", "--output", "json"])
        .output()
        .unwrap()
        .status
        .code()
        .unwrap_or(-1);
    assert!(
        accepted(code_json),
        "issue list --json rejected by clap (exit {code_json})"
    );
    assert!(
        accepted(code_output),
        "issue list --output json rejected by clap (exit {code_output})"
    );
    assert_eq!(
        code_json, code_output,
        "--json (exit {code_json}) and --output json (exit {code_output}) should produce the same exit code"
    );
}
