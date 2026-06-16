//! End-to-end characterization tests driving the `calc` binary through its CLI.
//!
//! These lock in the *current* observable behaviour (lex -> parse -> eval ->
//! format) so dependency upgrades and refactors can't silently change output.
//! They are intentionally high-level — see CLAUDE.md "Testing strategy".
//!
//! The currency path is excluded because it depends on the network / a live
//! MNB feed; see the ignored `currency_smoke` test at the bottom.

use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

/// Run the built binary in one-shot mode with a single expression argument and
/// return its trimmed stdout. CALC_CONFIG is pointed at a nonexistent path so
/// the binary always uses FormatOptions::default(), independent of any config
/// file on the developer's machine.
fn eval(expr: &str) -> String {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let output = Command::new(env!("CARGO_BIN_EXE_calc"))
        .arg(expr)
        .env("CALC_CONFIG", tmp.path().join("nonexistent.toml"))
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

/// Like `eval_repl` but with an explicit CALC_CONFIG path (for --global tests).
fn eval_repl_cfg(input: &str, config_path: &Path) -> String {
    let home = tempfile::tempdir().expect("create temp home dir");
    let mut child = Command::new(env!("CARGO_BIN_EXE_calc"))
        .env("HOME", home.path())
        .env_remove("XDG_CACHE_HOME")
        .env_remove("XDG_CONFIG_HOME")
        .env("CALC_CONFIG", config_path)
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
        ("0", "0"),
        ("2 + 2", "4"),
        ("10 - 3", "7"),
        ("6 * 7", "42"),
        ("-5 + 3", "-2"),
        ("-(2+3)", "-5"),
        // large negatives: no k/m suffix (those are opt-in, plan 4)
        ("-1000000", "-1000000"),
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
        ("1/2 + 1/3", "0.8333…"),
        ("1/3", "0.3333…"),
        ("10 / 3", "3.3333…"),
        // negative rational
        ("-1/3", "-0.3333…"),
        // exact results demote back to Int
        ("6 / 3", "2"),
        ("1/3 * 3", "1"),
    ]);
}

#[test]
fn float_results() {
    check(&[
        ("2^0.5", "1.4142…"),
        ("-2^0.5", "-1.4142…"),
        // whole-valued floats render as plain integers
        ("4^0.5", "2"),
        ("0.0", "0"),
        // IEEE 754 imprecision is visible via the … marker
        ("0.1 + 0.2", "0.3000…"),
    ]);
}

#[test]
fn integer_suffix_parsing() {
    // k and m are lexer suffixes for ×1000 / ×1000000; result is a plain integer
    check(&[
        ("3k", "3000"),
        ("3k * 2", "6000"),
        ("2m", "2000000"),
        ("1000000", "1000000"),
        ("1234567", "1234567"),
    ]);
}

#[test]
fn length_conversions() {
    check(&[
        ("5 m to cm", "500 cm"),
        ("1 km to m", "1000 m"),
        ("100 cm to m", "1 m"),
        // bidirectional: small exact result
        ("1 m to km", "0.001 km"),
        ("1 inch to cm", "2.54 cm"),
        ("1 ft to inch", "12 \""),
        ("1 mi to km", "1.6093… km"),
    ]);
}

#[test]
fn mass_volume_time_conversions() {
    check(&[
        ("1 gallon to l", "3.7854… l"),
        ("1 lb to kg", "0.4536… kg"),
        ("2 kg to g", "2000 g"),
        ("60 min to h", "1 h"),
        ("1 h to s", "3600 s"),
    ]);
}

#[test]
fn temperature_conversions() {
    check(&[
        ("95 f to c", "35 C"),
        ("0 c to f", "32 F"),
        ("100 c to f", "212 F"),
        // -40 is the unique crossover where °C == °F;
        // parenthesise so the unit attaches before conversion (not after negation)
        ("(-40) c to f", "-40 F"),
        ("(-40) f to c", "-40 C"),
    ]);
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
        ("5 m - 3 m", "2 m"),
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
    // incomplete expression (trailing operator)
    assert_eq!(eval("1 m +"), "");
}

#[test]
fn repl_evaluates_lines_and_continues_incomplete_input() {
    // three complete entries, then an expression split across two lines (the
    // open paren keeps the parser waiting until it is closed).
    let out = eval_repl("2 + 2\n1/2 + 1/3\n(2 +\n3)\n");
    assert_eq!(out, "4\n0.8333…\n5");
}

/// Run the binary in one-shot mode with custom env overrides. Returns
/// `(success, stdout, stderr)`.
fn eval_with_env(expr: &str, set: &[(&str, &str)], unset: &[&str]) -> (bool, String, String) {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_calc"));
    cmd.arg(expr);
    for (k, v) in set {
        cmd.env(k, v);
    }
    for k in unset {
        cmd.env_remove(k);
    }
    let output = cmd.output().expect("failed to run calc binary");
    let stdout = String::from_utf8(output.stdout)
        .expect("utf-8")
        .trim_end()
        .to_string();
    let stderr = String::from_utf8(output.stderr)
        .expect("utf-8")
        .trim_end()
        .to_string();
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
    let dir = tempfile::tempdir().expect("create temp dir");
    let missing = dir.path().join("nonexistent.toml"); // deliberately not created
    let (ok, out, _) = eval_with_env("2+2", &[("CALC_CONFIG", missing.to_str().unwrap())], &[]);
    assert!(
        ok,
        "missing CALC_CONFIG path should use defaults, not error"
    );
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
    assert!(
        config_path.exists(),
        "bootstrap should have created {config_path:?}"
    );
    let content = std::fs::read_to_string(&config_path).expect("read bootstrapped config");
    assert!(
        content.contains("[format]"),
        "bootstrapped file should contain a [format] section, got: {content:?}"
    );
}

/// Run with a custom config and return trimmed stdout. Pass a full TOML string
/// (e.g. `"[format]\nrepr = \"sci\""`) or `""` for defaults.
fn eval_with_format_config(expr: &str, config_toml: &str) -> String {
    let dir = tempfile::tempdir().expect("create temp dir");
    let conf = dir.path().join("conf.toml");
    std::fs::write(&conf, config_toml).expect("write config");
    let (ok, out, _) = eval_with_env(expr, &[("CALC_CONFIG", conf.to_str().unwrap())], &[]);
    assert!(ok, "calc failed for {expr:?} with config {config_toml:?}");
    out
}

#[test]
fn format_fixed_precision() {
    assert_eq!(
        eval_with_format_config("1/3", "[format.float]\nprecision = 2"),
        "0.33…"
    );
    assert_eq!(
        eval_with_format_config("1/2", "[format.float]\nprecision = 2"),
        "0.5"
    );
    assert_eq!(
        eval_with_format_config("1/3", "[format.float]\nprecision = 6"),
        "0.333333…"
    );
}

#[test]
fn format_scientific_thresholds() {
    // default (float repr, sci_upgrade = true): extreme values auto-upgrade to sci
    assert_eq!(eval_with_format_config("1.5e-8", ""), "1.5e-8");
    assert_eq!(eval_with_format_config("1500000.5", ""), "1.5000…e6");
    // fixed repr: always fixed-point, no auto-upgrade regardless of magnitude
    assert_eq!(
        eval_with_format_config("1500000.5", "[format]\nrepr = \"fixed\""),
        "1500000.5"
    );
}

#[test]
fn format_rational_mode() {
    assert_eq!(
        eval_with_format_config("1/3", "[format]\nrepr = \"rational\""),
        "1/3"
    );
    assert_eq!(
        eval_with_format_config("5/6", "[format]\nrepr = \"rational\""),
        "5/6"
    );
    // whole-valued rational demotes to Int — printed as integer, not a/b
    assert_eq!(
        eval_with_format_config("1/2 + 1/2", "[format]\nrepr = \"rational\""),
        "1"
    );
}

#[test]
fn format_int_scientific() {
    // integers: scientific only when opted in above threshold
    assert_eq!(
        eval_with_format_config(
            "1000000000000000",
            "[format.int]\nsci_upgrade = true\nsci_upgrade_upper = 1e12"
        ),
        "1e15"
    );
    // below threshold: plain
    assert_eq!(
        eval_with_format_config(
            "999",
            "[format.int]\nsci_upgrade = true\nsci_upgrade_upper = 1e12"
        ),
        "999"
    );
}

#[test]
fn repl_error_recovery() {
    // An eval error is printed but the REPL continues accepting input.
    let out = eval_repl("1/0\n2 + 2\n");
    assert_eq!(out, "Division by zero\n4");
}

#[test]
fn format_sci_precision() {
    // sci precision controls mantissa decimal places in scientific mode
    assert_eq!(
        eval_with_format_config("2250000.75", "[format.sci]\nprecision = 2"),
        "2.25…e6"
    );
    assert_eq!(
        eval_with_format_config("2250000.75", "[format.sci]\nprecision = 6"),
        "2.250001…e6"
    );
    // default (sci precision = 4) for reference
    assert_eq!(eval_with_format_config("2250000.75", ""), "2.2500…e6");
}

#[test]
fn format_operator_rational() {
    assert_eq!(eval("1/3 | rat"), "1/3");
    assert_eq!(eval("1/3 | rational"), "1/3");
    assert_eq!(eval("5/6 | rat"), "5/6");
    // whole-valued result: demotes to Int, printed without fraction
    assert_eq!(eval("1/2 + 1/2 | rat"), "1");
}

#[test]
fn format_operator_fixed() {
    assert_eq!(eval("1/3 | fixed 6"), "0.333333\u{2026}");
    assert_eq!(eval("1500000.5 | fixed"), "1500000.5");
    assert_eq!(eval("1/3 | fixed"), "0.3333\u{2026}");
}

#[test]
fn format_operator_sci() {
    assert_eq!(eval("1/1000000 | sci"), "1e-6");
    assert_eq!(eval("1/3 | sci 2"), "3.33\u{2026}e-1");
}

#[test]
fn format_operator_financial() {
    assert_eq!(eval("1234567 | fin"), "1.23\u{2026}m");
    assert_eq!(eval("1234567 | fin 0"), "1\u{2026}m");
    assert_eq!(eval("1234567 | financial"), "1.23\u{2026}m");
    // negative values preserve sign
    assert_eq!(eval("-1234567 | fin"), "-1.23\u{2026}m");
    // small values: no suffix
    assert_eq!(eval("42 | fin"), "42.00");
}

#[test]
fn format_operator_float() {
    // float with sci_upgrade: large values auto-upgrade to sci
    assert_eq!(eval("1500000.5 | float"), "1.5000\u{2026}e6");
    // float is also the default, so same as no spec for non-extreme values
    assert_eq!(eval("1/3 | float"), "0.3333\u{2026}");
}

#[test]
fn format_operator_with_unit() {
    // the | spec applies to the number part; the unit is still rendered
    assert_eq!(eval("1000000 m | sci"), "1e6 m");
}

#[test]
fn format_operator_unknown_produces_no_output() {
    // unknown formatter name: expression fails to parse, no output
    assert_eq!(eval("1/3 | bogus"), "");
}

#[test]
fn ans_keyword() {
    // one-shot: ans initializes to 0
    check(&[("ans", "0"), ("ans + 1", "1"), ("ans * 5", "0")]);

    // REPL: ans updates after each successful eval
    let out = eval_repl("5\nans + 3\nans * 2\n");
    assert_eq!(out, "5\n8\n16");

    // REPL: ans carries units
    let out = eval_repl("5 m\nans to cm\n");
    assert_eq!(out, "5 m\n500 cm");

    // REPL: eval errors do not update ans
    let out = eval_repl("7\n1/0\nans\n");
    assert_eq!(out, "7\nDivision by zero\n7");

    // REPL: ans | fmt reformats the previous result (primary use case)
    let out = eval_repl("1/2 + 1/3\nans | rat\n");
    assert_eq!(out, "0.8333\u{2026}\n5/6");
}

#[test]
fn currency_static_provider() {
    const CONFIG: &str = "[currency]\nprovider = \"static\"\n\n[currency.static]\n\"EUR/USD\" = 1.08\n\"USD/HUF\" = 360.0\n";
    // direct lookup: configured pairs work exactly
    assert_eq!(eval_with_format_config("100 EUR to USD", CONFIG), "108 USD");
    assert_eq!(eval_with_format_config("1 USD to HUF", CONFIG), "360 HUF");
    // inverse not configured → conversion error (no auto-fill)
    assert_eq!(
        eval_with_format_config("100 USD to EUR", CONFIG),
        "Conversion error"
    );
}

/// Currency conversion hits the live MNB feed (or a same-day cache) and is not
#[test]
fn config_show_all_defaults() {
    let out = eval_repl("/config\n");
    assert!(
        out.contains("format.repr = float"),
        "missing format.repr: {out:?}"
    );
    assert!(
        out.contains("format.float.precision = 4"),
        "missing float.precision: {out:?}"
    );
    assert!(
        out.contains("format.float.sci_upgrade = true"),
        "missing sci_upgrade: {out:?}"
    );
    assert!(
        out.contains("format.sci.precision = 4"),
        "missing sci.precision: {out:?}"
    );
    assert!(
        out.contains("format.fin.precision = 2"),
        "missing fin.precision: {out:?}"
    );
    assert!(
        out.contains("format.int.sci_upgrade = false"),
        "missing int.sci_upgrade: {out:?}"
    );
    assert!(
        out.contains("currency.provider = mnb"),
        "missing currency.provider: {out:?}"
    );
}

#[test]
fn config_show_single_key() {
    let out = eval_repl("/config format.float.precision\n");
    assert_eq!(out.trim(), "format.float.precision = 4");
}

#[test]
fn config_unknown_key_lists_valid() {
    let out = eval_repl("/config format.nope\n");
    assert!(out.contains("unknown key"), "expected error: {out:?}");
    assert!(
        out.contains("format.float.precision"),
        "should list valid keys: {out:?}"
    );
}

#[test]
fn meta_command_does_not_break_subsequent_expression() {
    let out = eval_repl("/config format.float.precision\n1 + 1\n");
    assert!(
        out.contains("format.float.precision = 4"),
        "config output missing: {out:?}"
    );
    assert!(out.contains('2'), "expression result missing: {out:?}");
}

#[test]
fn config_set_session_prints_new_value() {
    let out = eval_repl("/config format.float.precision 6\n");
    assert!(
        out.contains("format.float.precision = 6"),
        "set output: {out:?}"
    );
}

#[test]
fn config_set_session_affects_subsequent_expression() {
    let out = eval_repl("/config format.float.precision 6\n1/3\n");
    assert!(
        out.contains("format.float.precision = 6"),
        "set output missing: {out:?}"
    );
    assert!(out.contains("0.333333"), "expected 6 dp result: {out:?}");
}

#[test]
fn config_set_invalid_value_prints_error() {
    let out = eval_repl("/config format.float.precision notanumber\n");
    assert!(
        out.contains("expected integer"),
        "expected parse error: {out:?}"
    );
}

#[test]
fn config_set_unknown_key_prints_error() {
    let out = eval_repl("/config format.nonexistent 42\n");
    assert!(
        out.contains("unknown key"),
        "expected unknown-key error: {out:?}"
    );
    assert!(
        out.contains("format.float.precision"),
        "should list valid keys: {out:?}"
    );
}

#[test]
fn config_set_repr_enum() {
    let out = eval_repl("/config format.repr fixed\n");
    assert_eq!(out.trim(), "format.repr = fixed");
}

#[test]
fn config_global_persists_to_file() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let path = dir.path().join("conf.toml");

    // First session: set global
    let out = eval_repl_cfg("/config global format.float.precision 6\n", &path);
    assert!(
        out.contains("format.float.precision = 6"),
        "set output: {out:?}"
    );

    // File must exist and contain the new value
    let content = fs::read_to_string(&path).expect("config file not written");
    assert!(
        content.contains("precision = 6"),
        "config file content: {content:?}"
    );

    // Second session starting from the written file must read the persisted value
    let out2 = eval_repl_cfg("/config format.float.precision\n", &path);
    assert!(
        out2.contains("format.float.precision = 6"),
        "second session: {out2:?}"
    );
}

/// deterministic, so it is excluded from the default run. Execute explicitly
/// with `cargo test -- --ignored` when a network check is wanted.
#[test]
#[ignore = "requires network / live exchange rates"]
fn currency_smoke() {
    let out = eval("100 EUR to USD");
    assert!(out.ends_with("USD"), "unexpected output: {out:?}");
}
