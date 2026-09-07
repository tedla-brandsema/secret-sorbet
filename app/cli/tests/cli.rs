use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;

const PASSPHRASE: &str = "long private passphrase";
const PASSWORD: &str = "jSOua2Pe8n-xhU-kui9#";

#[test]
fn generates_the_checked_in_compatibility_vector() {
    cargo_bin_cmd!("seso")
        .args([
            "generate",
            "--service",
            "github.com",
            "--username",
            "ted@example.com",
            "--passphrase-stdin",
            "--print",
        ])
        .write_stdin(format!("{PASSPHRASE}\n"))
        .assert()
        .success()
        .stdout(format!("{PASSWORD}\n"))
        .stderr("");
}

#[test]
fn verbose_diagnostics_do_not_expose_sensitive_values() {
    let output = cargo_bin_cmd!("seso")
        .args([
            "--verbose",
            "generate",
            "--service",
            "github.com",
            "--username",
            "ted@example.com",
            "--counter",
            "7",
            "--passphrase-stdin",
            "--print",
        ])
        .write_stdin(format!("{PASSPHRASE}\n"))
        .output()
        .expect("CLI must execute");

    assert!(output.status.success());
    let generated = String::from_utf8(output.stdout)
        .expect("generated output must be UTF-8")
        .trim_end()
        .to_owned();
    let diagnostics = String::from_utf8(output.stderr).expect("diagnostics must be UTF-8");
    for sensitive in [
        PASSPHRASE,
        &generated,
        "github.com",
        "ted@example.com",
        "counter=7",
    ] {
        assert!(
            !diagnostics.contains(sensitive),
            "diagnostics exposed sensitive input: {sensitive}"
        );
    }
    assert!(diagnostics.contains("generation_started"));
    assert!(diagnostics.contains("generation_completed"));
}

#[test]
fn invalid_service_fails_without_writing_a_password() {
    cargo_bin_cmd!("seso")
        .args([
            "generate",
            "--service",
            "not-a-domain",
            "--username",
            "ted@example.com",
            "--passphrase-stdin",
        ])
        .write_stdin(format!("{PASSPHRASE}\n"))
        .assert()
        .failure()
        .stdout("")
        .stderr(predicate::str::contains(
            "error: password generation failed: the service must be a domain name without whitespace",
        ))
        .stderr(predicate::str::contains(PASSPHRASE).not());
}

#[test]
fn help_documents_the_secure_input_modes() {
    cargo_bin_cmd!("seso")
        .args(["generate", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--passphrase-stdin"))
        .stdout(predicate::str::contains("--print"))
        .stdout(predicate::str::contains("-i, --interactive"))
        .stdout(predicate::str::contains("--service <DOMAIN>"))
        .stdout(predicate::str::contains("--username <USERNAME>"));
}

#[test]
fn interactive_mode_conflicts_with_explicit_inputs() {
    cargo_bin_cmd!("seso")
        .args(["generate", "--interactive", "--service", "example.com"])
        .assert()
        .failure()
        .code(2)
        .stdout("")
        .stderr(predicate::str::contains("cannot be used with"))
        .stderr(predicate::str::contains("--interactive"))
        .stderr(predicate::str::contains("--service <DOMAIN>"));
}
