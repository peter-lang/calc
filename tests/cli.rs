//! End-to-end characterization tests driving the `calc` binary through its CLI.
//!
//! These lock in the *current* observable behaviour (lex -> parse -> eval ->
//! format) so dependency upgrades and refactors can't silently change output.
//! They are intentionally high-level — see CLAUDE.md "Testing strategy".
//!
//! The currency path is excluded because it depends on the network / a live
//! MNB feed; see the ignored `currency_smoke` test at the bottom.

use std::io::Write;
use std::process::{Command, Stdio};

/// Run the built binary in one-shot mode with a single expression argument and
/// return its trimmed stdout.
fn eval(expr: &str) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_calc"))
        .arg(expr)
        .output()
        .expect("failed to run calc binary");
    assert!(
        output.status.success(),
        "calc exited with failure for {expr:?}"
    );
    String::from_utf8(output.stdout)
        .expect("stdout was not utf-8")
        .trim_end()
        .to_string()
}

/// Drive the interactive REPL: feed `input` on stdin (each line is one REPL
/// entry) and return trimmed stdout. A fresh temp dir stands in for HOME so
/// the real history/cache/config files are never touched; it is deleted on drop.
fn eval_repl(input: &str) -> String {
    let home = tempfile::tempdir().expect("create temp home dir");
    let mut child = Command::new(env!("CARGO_BIN_EXE_calc"))
        .env("HOME", home.path())
        .env_remove("XDG_CACHE_HOME")
        .env_remove("XDG_CONFIG_HOME")
        .env_remove("CALC_CONFIG")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to spawn calc REPL");
    child
        .stdin
        .take()
        .expect("no stdin")
        .write_all(input.as_bytes())
        .expect("failed to write stdin");
    let output = child.wait_with_output().expect("failed to wait for REPL");
    // home dropped here — temp dir cleaned up
    String::from_utf8(output.stdout)
        .expect("stdout was not utf-8")
        .trim_end()
        .to_string()
}

/// Assert every `(input, expected stdout)` pair.
fn check(cases: &[(&str, &str)]) {
    for (input, expected) in cases {
        assert_eq!(&eval(input), expected, "input: {input:?}");
    }
}

#[test]
fn integer_arithmetic() {
    check(&[
        ("2 + 2", "4"),
        ("10 - 3", "7"),
        ("6 * 7", "42"),
        ("-5 + 3", "-2"),
        ("-(2+3)", "-5"),
    ]);
}

#[test]
fn operator_precedence_and_associativity() {
    check(&[
        ("2 + 3 * 4", "14"),
        ("(2 + 3) * 4", "20"),
        ("2^10", "1024"),
        ("2 ** 10", "1024"),
        // exponentiation is right-associative: 2^(2^3) = 2^8
        ("2^2^3", "256"),
    ]);
}

#[test]
fn rational_arithmetic_stays_exact() {
    check(&[
        ("7 / 2", "3.5"),
        ("1/2 + 1/2", "1"),
        ("1/2 + 1/3", "0.833333…"),
        ("1/3", "0.333333…"),
        ("10 / 3", "3.333…"),
    ]);
}

#[test]
fn float_results() {
    check(&[("2^0.5", "1.414…"), ("0.5", "0.5")]);
}

#[test]
fn number_formatting_suffixes_and_scientific() {
    check(&[
        ("3000", "3k"),
        ("1500", "1500"),
        ("3k * 2", "6k"),
        ("2m", "2m"),
        ("1000000", "1m"),
        ("1234567", "1.235m"),
        ("1e9", "1.000000e9"),
        ("1e12", "1.000000e12"),
    ]);
}

#[test]
fn length_conversions() {
    check(&[
        ("5 m to cm", "500 cm"),
        ("1 km to m", "1k m"),
        ("100 cm to m", "1 m"),
        ("1 inch to cm", "2.54 cm"),
        ("1 ft to inch", "12 \""),
        ("1 mi to km", "1.609… km"),
    ]);
}

#[test]
fn mass_volume_time_conversions() {
    check(&[
        ("1 gallon to l", "3.785… l"),
        ("1 lb to kg", "0.453592… kg"),
        ("2 kg to g", "2k g"),
        ("60 min to h", "1 h"),
    ]);
}

#[test]
fn temperature_conversions() {
    check(&[("95 f to c", "35 C"), ("0 c to f", "32 F")]);
}

#[test]
fn compound_quantities() {
    // adjacent quantities are summed within a compound group: m+cm, ft+in, h/min/s
    check(&[
        ("5 m 10 cm to cm", "510 cm"),
        ("5 ft 11 in to cm", "180.34 cm"),
        ("1 h 30 min to min", "90 min"),
        ("1 h 30 min 15 s to s", "5415 s"),
    ]);
}

#[test]
fn unit_attaches_to_parenthesized_value() {
    // a unit right after `( … )` attaches to the value (implicit multiplication)
    check(&[("(2*3) m to cm", "600 cm"), ("(2+1) m", "3 m")]);
}

#[test]
fn units_with_arithmetic() {
    check(&[
        ("1 m", "1 m"),
        // a bare unit means quantity 1
        ("m", "1 m"),
        ("3 m * 2", "6 m"),
        ("6 m / 2", "3 m"),
    ]);
}

#[test]
fn error_messages() {
    check(&[
        ("1/0", "Division by zero"),
        ("5 m + 3 kg", "Different unit types"),
        ("5 m to kg", "Different unit types"),
        // multiplying/dividing two united values is rejected (no derived units)
        ("3 m * 2 m", "Cannot operate with units"),
        ("100 km / 2 h", "Cannot operate with units"),
        ("2 ^ 3 m", "Exponent cannot have a unit"),
        // raising a united value to a power would be a derived unit — not yet
        ("(2 m)^2", "Cannot operate with units"),
    ]);
}

#[test]
fn unparseable_input_produces_no_output() {
    // `%` is tokenized but has no grammar rule, so the expression fails to parse
    // and nothing is printed. Documents current behaviour.
    assert_eq!(eval("10 % 3"), "");
    // mass isn't a compound group, so `5 kg 10 g` doesn't parse
    assert_eq!(eval("5 kg 10 g"), "");
}

#[test]
fn repl_evaluates_lines_and_continues_incomplete_input() {
    // three complete entries, then an expression split across two lines (the
    // open paren keeps the parser waiting until it is closed).
    let out = eval_repl("2 + 2\n1/2 + 1/3\n(2 +\n3)\n");
    assert_eq!(out, "4\n0.833333…\n5");
}

/// Run the binary in one-shot mode with custom env overrides. Returns
/// `(success, stdout, stderr)`.
fn eval_with_env(
    expr: &str,
    set: &[(&str, &str)],
    unset: &[&str],
) -> (bool, String, String) {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_calc"));
    cmd.arg(expr);
    for (k, v) in set {
        cmd.env(k, v);
    }
    for k in unset {
        cmd.env_remove(k);
    }
    let output = cmd.output().expect("failed to run calc binary");
    let stdout = String::from_utf8(output.stdout).expect("utf-8").trim_end().to_string();
    let stderr = String::from_utf8(output.stderr).expect("utf-8").trim_end().to_string();
    (output.status.success(), stdout, stderr)
}

#[test]
fn config_valid_calc_config_env() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let conf = dir.path().join("conf.toml");
    // an empty file is valid TOML and should fall back to defaults
    std::fs::write(&conf, "").expect("write empty config");
    let (ok, out, _) = eval_with_env("2+2", &[("CALC_CONFIG", conf.to_str().unwrap())], &[]);
    assert!(ok, "calc should succeed with an empty config file");
    assert_eq!(out, "4");
}

#[test]
fn config_missing_calc_config_path_uses_defaults() {
    let (ok, out, _) = eval_with_env(
        "2+2",
        &[("CALC_CONFIG", "/tmp/calc-this-file-does-not-exist-xyzzy.toml")],
        &[],
    );
    assert!(ok, "missing CALC_CONFIG path should use defaults, not error");
    assert_eq!(out, "4");
}

#[test]
fn config_malformed_calc_config_errors() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let conf = dir.path().join("conf.toml");
    std::fs::write(&conf, "[[not valid toml !!!").expect("write bad config");
    let (ok, _, err) = eval_with_env("2+2", &[("CALC_CONFIG", conf.to_str().unwrap())], &[]);
    assert!(!ok, "malformed config should cause non-zero exit");
    // Rust's main() Result prints via Debug, so the variant name appears.
    assert!(
        err.contains("ConfigError"),
        "stderr should mention ConfigError, got: {err:?}"
    );
}

#[test]
fn config_first_run_bootstrap_creates_file() {
    let home = tempfile::tempdir().expect("create temp home dir");
    let (ok, out, _) = eval_with_env(
        "1+1",
        &[("HOME", home.path().to_str().unwrap())],
        &["CALC_CONFIG", "XDG_CONFIG_HOME"],
    );
    assert!(ok, "first-run should succeed");
    assert_eq!(out, "2");

    // The bootstrap should have created conf.toml under the XDG config dir.
    let config_path = home.path().join(".config").join("calc").join("conf.toml");
    assert!(config_path.exists(), "bootstrap should have created {config_path:?}");
}

/// Currency conversion hits the live MNB feed (or a same-day cache) and is not
/// deterministic, so it is excluded from the default run. Execute explicitly
/// with `cargo test -- --ignored` when a network check is wanted.
#[test]
#[ignore = "requires network / live exchange rates"]
fn currency_smoke() {
    let out = eval("100 EUR to USD");
    assert!(out.ends_with("USD"), "unexpected output: {out:?}");
}
