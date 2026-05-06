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
fn project_list_accepts_json_flag() {
    let code = linear()
        .args(["project", "list", "--json"])
        .output()
        .unwrap()
        .status
        .code()
        .unwrap_or(-1);
    assert!(
        accepted(code),
        "project list --json rejected by clap (exit {code})"
    );
}

#[test]
fn project_get_accepts_json_flag() {
    let code = linear()
        .args(["project", "get", "some-slug", "--json"])
        .output()
        .unwrap()
        .status
        .code()
        .unwrap_or(-1);
    assert!(
        accepted(code),
        "project get --json rejected by clap (exit {code})"
    );
}

#[test]
fn project_create_accepts_json_flag() {
    let code = linear()
        .args([
            "project", "create", "--name", "p", "--team", "ENG", "--json",
        ])
        .output()
        .unwrap()
        .status
        .code()
        .unwrap_or(-1);
    assert!(
        accepted(code),
        "project create --json rejected by clap (exit {code})"
    );
}

#[test]
fn project_update_accepts_json_flag() {
    let code = linear()
        .args(["project", "update", "some-slug", "--name", "p", "--json"])
        .output()
        .unwrap()
        .status
        .code()
        .unwrap_or(-1);
    assert!(
        accepted(code),
        "project update --json rejected by clap (exit {code})"
    );
}

#[test]
fn project_archive_accepts_json_flag() {
    let code = linear()
        .args(["project", "archive", "some-slug", "--json"])
        .output()
        .unwrap()
        .status
        .code()
        .unwrap_or(-1);
    assert!(
        accepted(code),
        "project archive --json rejected by clap (exit {code})"
    );
}
