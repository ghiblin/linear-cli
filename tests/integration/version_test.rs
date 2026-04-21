use std::process::Command;

#[test]
fn version_outputs_json_with_required_fields() {
    let binary = env!("CARGO_BIN_EXE_linear");
    let output = Command::new(binary)
        .arg("--version")
        .output()
        .expect("failed to spawn binary");

    let stdout = String::from_utf8(output.stdout).unwrap();
    let json: serde_json::Value =
        serde_json::from_str(stdout.trim()).expect("--version output should be valid JSON");
    assert!(json.get("version").is_some(), "missing 'version' field");
    assert!(
        json.get("api_schema").is_some(),
        "missing 'api_schema' field"
    );
}

#[test]
fn version_output_snapshot() {
    let output = Command::new(env!("CARGO_BIN_EXE_linear"))
        .arg("--version")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    insta::assert_snapshot!(stdout.trim());
}
