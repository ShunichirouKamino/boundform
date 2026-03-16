use std::process::Command;

fn boundform_cmd() -> Command {
    Command::new(env!("CARGO_BIN_EXE_boundform"))
}

#[test]
fn test_validate_matching_config() {
    let output = boundform_cmd()
        .args(["--config", "tests/fixtures/boundform.yml"])
        .output()
        .expect("failed to execute");

    assert!(
        output.status.success(),
        "should exit with 0 when all constraints match"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("register"), "should mention form id");
    assert!(stdout.contains("checks passed"), "should report all passed");
}

#[test]
fn test_validate_matching_config_json() {
    let output = boundform_cmd()
        .args([
            "--config",
            "tests/fixtures/boundform.yml",
            "--format",
            "json",
        ])
        .output()
        .expect("failed to execute");

    assert!(output.status.success(), "should exit with 0");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("should be valid JSON");
    assert!(parsed["pages"].is_array());
    assert_eq!(parsed["pages"][0]["form_results"][0]["form_id"], "register");
    assert_eq!(parsed["pages"][0]["form_results"][0]["found"], true);
}

#[test]
fn test_validate_mismatch_config() {
    let output = boundform_cmd()
        .args(["--config", "tests/fixtures/boundform_mismatch.yml"])
        .output()
        .expect("failed to execute");

    assert!(!output.status.success(), "should exit with 1 on mismatch");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("maxlength"),
        "should report maxlength mismatch"
    );
    assert!(
        stdout.contains("minlength"),
        "should report minlength mismatch"
    );
}

#[test]
fn test_validate_nonexistent_config() {
    let output = boundform_cmd()
        .args(["--config", "tests/fixtures/nonexistent.yml"])
        .output()
        .expect("failed to execute");

    assert!(!output.status.success(), "should exit with error");
}
