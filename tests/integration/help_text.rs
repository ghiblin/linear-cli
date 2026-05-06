use std::process::Command;

fn help_output(args: &[&str]) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_linear"))
        .args(args)
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    stdout + &stderr
}

#[test]
fn issue_list_help_shows_json_flag() {
    let text = help_output(&["issue", "list", "--help"]);
    assert!(
        text.contains("--json"),
        "issue list --help does not mention --json:\n{text}"
    );
}

#[test]
fn project_list_help_shows_json_flag() {
    let text = help_output(&["project", "list", "--help"]);
    assert!(
        text.contains("--json"),
        "project list --help does not mention --json:\n{text}"
    );
}

#[test]
fn team_list_help_shows_json_flag() {
    let text = help_output(&["team", "list", "--help"]);
    assert!(
        text.contains("--json"),
        "team list --help does not mention --json:\n{text}"
    );
}
