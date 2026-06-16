use rustyline::completion::{Completer, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::history::DefaultHistory;
use rustyline::validate::Validator;
use rustyline::{CompletionType, Config as RlConfig, Context, Editor, Helper};

use crate::config::{self, FormatSpec};
use crate::error::CalcError;
use crate::files;
use crate::node::Node;
use crate::parser::lexer::Lexer;
use crate::parser::parser::Parser;
use crate::value::{format_value, Value};

fn has_children(candidate: &str) -> bool {
    let prefix = format!("{}.", candidate);
    config::REGISTRY.iter().any(|e| e.key.starts_with(&prefix))
}

/// Given a registry key and the prefix typed so far, return the completion
/// candidate up to the next dot boundary after the prefix, or the full key
/// for leaf nodes. Enables one-level-at-a-time hierarchical completion.
///
/// Examples (prefix → candidate):
///   ""              + "format.float.precision" → "format"
///   "format"        + "format.float.precision" → "format.float"
///   "format.float"  + "format.float.precision" → "format.float.precision"
fn next_level_completion<'a>(key: &'a str, prefix: &str) -> &'a str {
    let suffix = &key[prefix.len()..];
    if suffix.is_empty() {
        return key;
    }
    let (search, base) = if suffix.starts_with('.') {
        (&suffix[1..], prefix.len() + 1)
    } else {
        (suffix, prefix.len())
    };
    match search.find('.') {
        Some(i) => &key[..base + i],
        None => key,
    }
}

struct ReplHelper;

impl Completer for ReplHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        let before = &line[..pos];

        // Complete the command name itself: "/con" → "/config "
        if before.starts_with('/') && !before.contains(' ') {
            let candidates = ["/config"]
                .iter()
                .filter(|cmd| cmd.starts_with(before))
                .map(|cmd| Pair {
                    display: cmd.to_string(),
                    replacement: format!("{} ", cmd),
                })
                .collect();
            return Ok((0, candidates));
        }

        let (key_prefix, after_global) = if let Some(r) = before.strip_prefix("/config global ") {
            (r, true)
        } else if let Some(r) = before.strip_prefix("/config ") {
            (r, false)
        } else {
            return Ok((pos, vec![]));
        };
        if let Some((key, val_prefix)) = key_prefix.split_once(' ') {
            let completions = config::REGISTRY
                .iter()
                .find(|e| e.key == key)
                .map(|e| {
                    e.completions
                        .iter()
                        .filter(|v| v.starts_with(val_prefix))
                        .map(|v| Pair {
                            display: v.to_string(),
                            replacement: v.to_string(),
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            Ok((pos - val_prefix.len(), completions))
        } else {
            let mut seen = std::collections::HashSet::new();
            let mut candidates: Vec<Pair> = Vec::new();
            if !after_global && "global".starts_with(key_prefix) {
                seen.insert("global".to_string());
                candidates.push(Pair {
                    display: "global".to_string(),
                    replacement: "global ".to_string(),
                });
            }
            candidates.extend(
                config::REGISTRY
                    .iter()
                    .filter(|e| e.key.starts_with(key_prefix))
                    .filter_map(|e| {
                        let c = next_level_completion(e.key, key_prefix).to_string();
                        if !seen.insert(c.clone()) {
                            return None;
                        }
                        let (display, replacement) = if has_children(&c) {
                            (format!("{}.", c), format!("{}.", c))
                        } else {
                            (c.clone(), c)
                        };
                        Some(Pair {
                            display,
                            replacement,
                        })
                    }),
            );
            Ok((pos - key_prefix.len(), candidates))
        }
    }
}

impl Hinter for ReplHelper {
    type Hint = String;
    fn hint(&self, line: &str, pos: usize, _ctx: &Context<'_>) -> Option<String> {
        if pos < line.len() {
            return None;
        }
        let rest = line
            .strip_prefix("/config global ")
            .or_else(|| line.strip_prefix("/config "))?;
        if rest.contains(' ') {
            return None;
        }
        let entry = config::REGISTRY.iter().find(|e| e.key == rest)?;
        let cfg = config::current();
        Some(format!("  →  {}", (entry.get)(&cfg)))
    }
}

impl Highlighter for ReplHelper {}
impl Validator for ReplHelper {}
impl Helper for ReplHelper {}

fn handle_meta_command(line: &str) {
    let mut parts = line.splitn(2, ' ');
    let cmd = parts.next().unwrap_or("");
    let rest = parts.next().map(str::trim);
    match cmd {
        "/config" => match rest {
            None => {
                let cfg = config::current();
                for entry in config::REGISTRY {
                    println!("{} = {}", entry.key, (entry.get)(&cfg));
                }
            }
            Some(rest) => {
                let (global, rest) = match rest.strip_prefix("global") {
                    Some(after) if after.is_empty() || after.starts_with(' ') => {
                        let after = after.trim();
                        if after.is_empty() {
                            println!("usage: /config global <key> <value>");
                            return;
                        }
                        (true, after)
                    }
                    _ => (false, rest),
                };
                match rest.split_once(' ') {
                    None => {
                        let cfg = config::current();
                        match config::REGISTRY.iter().find(|e| e.key == rest) {
                            Some(entry) => println!("{} = {}", entry.key, (entry.get)(&cfg)),
                            None => {
                                let keys: Vec<_> = config::REGISTRY.iter().map(|e| e.key).collect();
                                println!("unknown key {rest:?}; valid keys: {}", keys.join(", "));
                            }
                        }
                    }
                    Some((key, val)) => {
                        let key = key.trim();
                        let val = val.trim();
                        match config::set_key(key, val) {
                            Ok(new_val) => {
                                println!("{key} = {new_val}");
                                if global {
                                    if let Err(e) = config::persist() {
                                        println!("error persisting config: {e}");
                                    }
                                }
                            }
                            Err(e) => println!("{e}"),
                        }
                    }
                }
            }
        },
        other => println!("unknown command {other:?}; available: /config"),
    }
}

/// Evaluate a parsed node, print its formatted result (or the error), and
/// return the value on success so callers can record it as `ans`.
fn evaluate_and_print(node: Node, spec: &Option<FormatSpec>) -> Option<Value> {
    match node.eval() {
        Ok(res) => {
            let opts = {
                let guard = config::current();
                match spec {
                    Some(s) => config::apply_spec(&guard.format, s),
                    None => guard.format.clone(),
                }
            };
            println!("{}", format_value(&res, &opts));
            Some(res)
        }
        Err(error) => {
            println!("{error}");
            None
        }
    }
}

/// Interactive REPL: read lines, dispatch `/`-meta-commands, otherwise feed the
/// parser and evaluate complete expressions.
pub fn run() -> Result<(), CalcError> {
    let path_cache_history = files::cache("history.txt")?;
    let lexer = Lexer::new();

    let rl_config = RlConfig::builder()
        .completion_type(CompletionType::List)
        .build();
    let mut rl = Editor::<ReplHelper, DefaultHistory>::with_config(rl_config)?;
    rl.set_helper(Some(ReplHelper));
    if path_cache_history.exists() {
        rl.load_history(&path_cache_history)?;
    }

    let mut parser = Parser::new();
    let mut line_buffer = String::new();
    loop {
        match rl.readline(if parser.is_empty() { ">> " } else { ".. " }) {
            Ok(line) => {
                if line.trim_start().starts_with('/') {
                    handle_meta_command(line.trim());
                    let _ = rl.add_history_entry(&line);
                    continue;
                }
                if !line_buffer.is_empty() {
                    line_buffer.push_str(" ");
                }
                line_buffer.push_str(line.as_str());
                parser.extend(lexer.parse(line.as_str()));

                if let Some((node, spec)) = parser.parse() {
                    if let Some(res) = evaluate_and_print(node, &spec) {
                        parser.set_ans(res);
                    }
                    let _ = rl.add_history_entry(line_buffer.as_str());
                    line_buffer.clear();
                    parser.reset();
                }
            }
            Err(ReadlineError::Interrupted | ReadlineError::Eof) => {
                if parser.is_empty() {
                    break;
                } else {
                    line_buffer.clear();
                    parser.reset();
                }
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    rl.save_history(&path_cache_history)?;
    Ok(())
}

/// One-shot evaluation of a single expression string (CLI argument mode).
pub fn run_once(input: &str) -> Result<(), CalcError> {
    let lexer = Lexer::new();
    let mut parser = Parser::new();
    parser.extend(lexer.parse(input));
    if let Some((node, spec)) = parser.parse() {
        evaluate_and_print(node, &spec);
    }
    Ok(())
}
