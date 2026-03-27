/// Test utilities for llguidance grammar testing.
///
/// This crate provides shared test infrastructure used by integration tests
/// in the `parser` crate. It is **not** intended as an example of how to use
/// llguidance in production. For that, see the `sample_parser` crate.
///
/// Key components:
/// - [`PARSER_FACTORY`]: A shared [`ParserFactory`] with the Phi-3.5-mini-instruct
///   tokenizer, configured for testing (ff_tokens + backtrack enabled, verbose logging).
/// - [`check_lark_grammar`] and friends: Verify that a grammar produces the expected
///   sequence of forced and generated tokens. Test traces are recorded by passing
///   `"test_trace": true` in the llguidance request.
/// - [`check_capture`]: Verify named captures in grammar output.
/// - [`lark_str_test`], [`json_schema_check`], etc.: Higher-level helpers for
///   validating strings/JSON against grammars and schemas.
use anyhow::Result;
use lazy_static::lazy_static;
use llguidance::{
    api::{GrammarInit, GrammarWithLexer, StopReason, TopLevelGrammar},
    earley::{SlicedBiasComputer, XorShift},
    toktrie::{bytes::limit_str, InferenceCapabilities, TokEnv, TokenId},
    Constraint, ParserFactory, TokenParser,
};
use serde_json::Value;

// ── Trace-based grammar checking ─────────────────────────────────────────────

/// Check that the grammar generates the expected output.
///
/// Output is a list of strings, each of which is a sequence of tokens.
/// Tokens in the string are separated with "‧".
/// Strings at even positions are "forced tokens", and strings at odd positions
/// are "generated tokens".
/// We check that the grammars forces the forced tokens (first of which is the
/// prompt), and that it allows in the mask the generated tokens.
///
/// These tests are "recorded" by passing "test_trace": true in the llguidance
/// request and post-processing.
fn check_grammar(
    factory: &ParserFactory,
    prompt_str: &str,
    grammar: TopLevelGrammar,
    output: &[&str],
    temp: f32,
) -> Constraint {
    let mut rnd = XorShift::new_str(&serde_json::to_string(&grammar).unwrap());

    let parser = factory.create_parser(grammar).unwrap();
    let can_rollback = parser.parser.grammar().lexer_spec().can_rollback();
    // let can_rollback = false;
    let mut parser2 = parser.deep_clone();
    let mut constraint = Constraint::new(parser);

    let tok_env = factory.tok_env();

    let prompt = tok_env.tokenize(prompt_str);
    let prompt2 = parser2.process_prompt(prompt.clone());
    let had_token_healing = prompt != prompt2;
    let prompt = constraint.process_prompt(prompt);
    check_eq(tok_env, "prompt", &prompt, output[0]);
    check_eq(tok_env, "prompt2", &prompt2, output[0]);

    let mut idx = 1;
    let mut gen_tokens = tokenize_trace(tok_env, output[idx]);
    let mut seen_temp = temp == 0.0;

    for step_idx in 0..200 {
        let res = constraint.compute_mask().unwrap().clone();

        if can_rollback {
            let mut n_tok = 0;
            loop {
                if step_idx == 0 && had_token_healing {
                    break;
                }
                eprintln!("\nRollback #{n_tok}");
                if parser2.check_stop().unwrap() {
                    break;
                }
                let m = parser2.compute_mask();
                if m.is_err() && parser2.stop_reason() == StopReason::NoExtensionBias {
                    break;
                }
                let m = m.unwrap();
                let tok = rnd.sample_from_vob(&m);
                let bt = parser2.consume_token(tok).unwrap();
                assert!(bt == 0);
                n_tok += 1;
                if tok == tok_env.tok_trie().eos_token() {
                    break;
                }
                if rnd.one_in(10) {
                    if rnd.one_in(2) {
                        let _ = parser2.compute_ff_tokens();
                    }
                    break;
                }
                let _ = parser2.compute_ff_tokens();
            }
            if n_tok > 0 {
                eprintln!("\nRun Rollback n_tok={n_tok}");
                parser2.rollback(n_tok).unwrap();
            }
            if let Some(m) = &res.sample_mask {
                eprintln!("\nRollback MASK");
                let m2 = parser2.compute_mask().unwrap();
                assert_eq!(m.len(), m2.len());
                for i in 0..m.len() {
                    if m[i] != m2[i] {
                        panic!(
                            "Mask mismatch at {}: {} rollback={}",
                            tok_env.tok_trie().token_dbg(i as u32),
                            m[i],
                            m2[i]
                        );
                    }
                }
            }
        }

        if let Some(t) = res.temperature {
            assert!(t == temp || t == 0.0, "Expected temperature {temp} got {t}");
            if t == temp {
                seen_temp = true;
            }
        }

        if res.is_stop() {
            assert!(idx >= output.len() - 1, "Expected more output at {idx}");
            assert!(
                gen_tokens.is_empty(),
                "Expected more tokens to generate; got stop"
            );
            break;
        }

        let mut bt: u32;
        let mut toks: Vec<TokenId>;

        if let Some(mask) = &res.sample_mask {
            if gen_tokens.is_empty() {
                panic!("No more tokens to generate");
            }

            loop {
                let (is_allowed, tok) = gen_tokens[0];
                if is_allowed {
                    break;
                }
                assert!(
                    !mask.is_allowed(tok),
                    "Token {} {} should not be allowed",
                    tok,
                    tok_env.tok_trie().token_dbg(tok)
                );
                assert!(
                    constraint.validate_tokens_raw(&[tok]).unwrap() == 0,
                    "Token {} {} should not validate",
                    tok,
                    tok_env.tok_trie().token_dbg(tok)
                );
                gen_tokens.remove(0);
            }

            let (_, tok) = gen_tokens[0];
            assert!(
                mask.is_allowed(tok),
                "Token {} {} should be allowed",
                tok,
                tok_env.tok_trie().token_dbg(tok)
            );

            let rest_allowed = gen_tokens
                .iter()
                .filter(|(is_allowed, _)| *is_allowed)
                .map(|(_, tok)| *tok)
                .collect::<Vec<_>>();

            let num_ok = constraint.validate_tokens_raw(&rest_allowed).unwrap();
            if num_ok < rest_allowed.len() {
                // figure out which, if any isn't allowed by masks
                for tok in &rest_allowed {
                    eprintln!(
                        "\nChecking token {} {}",
                        tok,
                        tok_env.tok_trie().token_dbg(*tok)
                    );
                    let mask = constraint.parser.compute_mask();
                    if mask.is_err() {
                        eprint!("Error computing mask: {mask:?}");
                        break;
                    }
                    let mask = mask.unwrap();
                    if !mask.is_allowed(*tok) {
                        eprintln!(
                            "Token {} {} not allowed by mask",
                            tok,
                            tok_env.tok_trie().token_dbg(*tok)
                        );
                    }
                    let r = constraint.parser.consume_token(*tok);
                    if r.is_err() {
                        eprint!("Error consuming token: {r:?}");
                        break;
                    }
                    let r = r.unwrap();
                    if r != 0 {
                        eprintln!(
                            "Token {} {} generated backtrack {}",
                            tok,
                            tok_env.tok_trie().token_dbg(*tok),
                            r
                        );
                    }
                }
                panic!(
                    "Expected {} tokens to be allowed; got {}; {}",
                    rest_allowed.len(),
                    num_ok,
                    tok_env.tok_trie().tokens_dbg(&rest_allowed)
                );
            }
            gen_tokens.remove(0);

            let res = constraint.commit_token(Some(tok)).unwrap();

            if can_rollback {
                let bt = parser2.consume_token(tok).unwrap();
                assert!(bt == 0);
                let mut ff = parser2.consume_ff_tokens().unwrap();
                ff.insert(0, tok);
                assert_eq!(ff, res.ff_tokens);
            }

            bt = res.backtrack;
            toks = res.ff_tokens.clone();
            if toks.is_empty() || toks[0] != tok {
                if idx + 1 < output.len() && output[idx + 1].starts_with("1↶") {
                    // fast-forward with fake backtrack
                    assert!(bt == 0 || res.ff_tokens.is_empty());
                    bt = 1;
                    // go to forced byte checking
                } else if toks.is_empty() {
                    panic!("Expected {tok}; got nothing");
                } else {
                    panic!("Expected token {} got {}", tok, toks[0]);
                }
            } else if toks.len() > 1 {
                // we got fast-forwarded to the next entry,
                // delete the generated tokens and leave the rest for forced
                // bytes checking below
                toks.remove(0);
                // go to forced byte checking
            } else {
                assert!(bt == 0);
                assert!(toks.len() == 1);
                continue; // normal path
            }
        } else {
            let res = constraint.commit_token(None).unwrap();
            bt = res.backtrack;
            toks = res.ff_tokens.clone();

            if can_rollback {
                let ff = parser2.consume_ff_tokens().unwrap();
                assert_eq!(ff, res.ff_tokens);
            }
        }

        // forced byte checking
        assert!(
            gen_tokens.is_empty(),
            "Expected more tokens to generate, got forced {}",
            tok_env.tok_trie().test_trace_tokens(&toks)
        );

        idx += 1;
        let mut expected = output[idx];
        if expected.contains("↶") {
            let parts: Vec<&str> = expected.split("↶").collect();
            assert!(parts.len() == 2);
            expected = parts[1];
            assert!(
                bt == parts[0].parse::<u32>().unwrap(),
                "Expected backtrack {} got {}",
                parts[0],
                bt
            );
        }
        check_eq(tok_env, &format!("step {idx}"), &toks, expected);
        idx += 1;
        if idx < output.len() {
            gen_tokens = tokenize_trace(tok_env, output[idx]);
        }
    }

    assert!(seen_temp, "Expected temperature {temp} not seen");

    constraint
}

fn check_eq(tok_env: &TokEnv, label: &str, tokens: &[TokenId], expected_tokens: &str) {
    let trie = tok_env.tok_trie();
    let actual_tokens = trie.test_trace_tokens(tokens);
    let expected_tokens = expected_tokens.replace("\n", "\\n");
    println!("Checking {label}: exp:{expected_tokens:?} got:{actual_tokens:?}");
    assert_eq!(actual_tokens, expected_tokens, "Tokens mismatch in {label}");
}

fn tokenize_trace(tok_env: &TokEnv, s: &str) -> Vec<(bool, TokenId)> {
    let trie = tok_env.tok_trie();
    println!("Tokenizing {s:?}");
    let mut result = Vec::new();
    if s.is_empty() {
        return result;
    }

    // Split by both ‧ and × to catch all tokens
    let words = s.split(['‧', '✖']).collect::<Vec<&str>>();
    let mut char_pos = 0;

    for word in words {
        if word.is_empty() {
            char_pos += 1;
            continue;
        }

        // Determine if this token started with ‧ (true) or × (false)
        let is_allowed = if char_pos > 0 {
            let char_before = s.chars().nth(char_pos - 1).unwrap_or('✖');
            char_before == '‧'
        } else {
            true // First token assumed to start with ‧
        };

        if word == "≺EOS≻" {
            result.push((is_allowed, trie.eos_token()));
        } else if let Some(t) = trie.get_special_token(word) {
            result.push((is_allowed, t));
        } else if word.starts_with("<[") && word.ends_with("]>") {
            let t = word[2..word.len() - 2].parse::<u32>().unwrap();
            assert!(t < trie.vocab_size() as u32);
            result.push((is_allowed, t));
        } else {
            let tt = trie.greedy_tokenize(word.as_bytes());
            assert!(
                tt.len() == 1,
                "Expected single token for {:?} got {}",
                word,
                trie.test_trace_tokens(&tt)
            );
            result.push((is_allowed, tt[0]));
        }

        char_pos += word.chars().count() + 1; // +1 for the separator
    }

    result
}

lazy_static! {
    static ref PARSER_FACTORY: ParserFactory = {
        let env =
            toktrie_hf_downloader::byte_tokenizer_from_name("microsoft/Phi-3.5-mini-instruct")
            .unwrap()
            .into_tok_env(Some(35000))
            .unwrap();
        let mut fact = ParserFactory::new(&env,
            InferenceCapabilities {
                ff_tokens: true, // can the engine append multiple tokens?
                backtrack: true, // can the engine remove generated tokens?
                conditional_ff_tokens: false, // not used
                fork: false,                  // not used
            }, &SlicedBiasComputer::general_slices()).unwrap();
        fact.set_stderr_log_level(2);
        fact.set_buffer_log_level(0);
        fact
    };
}

pub fn get_tok_env() -> &'static TokEnv {
    PARSER_FACTORY.tok_env()
}

pub fn get_parser_factory() -> &'static ParserFactory {
    &PARSER_FACTORY
}

pub fn check_lark_grammar_prompt(lark: &str, prompt_str: &str, output: &[&str]) -> Constraint {
    let grm = TopLevelGrammar::from_lark(lark.to_string());
    println!("\nChecking grammar:\n{lark}\nagainst: {output:?}");
    let temp = find_temperature(lark);
    check_grammar(&PARSER_FACTORY, prompt_str, grm, output, temp)
}

pub fn check_lark_grammar(lark: &str, output: &[&str]) -> Constraint {
    check_lark_grammar_prompt(lark, "", output)
}

fn find_temperature(lark: &str) -> f32 {
    lark.find("temperature=")
        .map(|i| {
            let i = i + "temperature=".len();
            let mut end = i;
            while end < lark.len()
                && (lark.as_bytes()[end].is_ascii_digit() || lark.as_bytes()[end] == b'.')
            {
                end += 1;
            }
            lark[i..end].parse::<f32>().unwrap()
        })
        .unwrap_or(0.0)
}

pub fn check_lark_grammar_nested(lark: &str, sub_lark: &str, output: &[&str]) -> Constraint {
    let temp = find_temperature(lark);
    let mut top_grm = TopLevelGrammar::from_lark(lark.to_string());
    let mut sub_grm = GrammarWithLexer::from_lark(sub_lark.to_string());
    sub_grm.name = Some("sub".to_string());
    top_grm.grammars.push(sub_grm);
    println!("\nChecking nested grammars:\n{lark}\nNested:\n{sub_lark}\nagainst: {output:?}");
    let r = check_grammar(&PARSER_FACTORY, "", top_grm, output, temp);

    if true {
        // also test the new syntax
        let lark2 = lark.replace("@sub", &format!("%lark {{\n{sub_lark}\n}}"));
        check_grammar(
            &PARSER_FACTORY,
            "",
            TopLevelGrammar::from_lark(lark2),
            output,
            temp,
        );
    }

    r
}

pub fn check_lark_json(lark: &str, json_schema: Value, output: &[&str]) -> Constraint {
    let temp = find_temperature(lark);
    let schema_str = serde_json::to_string_pretty(&json_schema).unwrap();
    let mut top_grm = TopLevelGrammar::from_lark(lark.to_string());
    let mut sub_grm = GrammarWithLexer::from_json_schema(json_schema);
    sub_grm.name = Some("sub".to_string());
    top_grm.grammars.push(sub_grm);
    println!("\nChecking lark+json:\n{lark}\nNested:\n{schema_str}\nagainst: {output:?}");
    check_grammar(&PARSER_FACTORY, "", top_grm, output, temp)
}

pub fn check_capture(c: &Constraint, name: &str, expected: &str) {
    if let Some(bytes) = c.parser.get_capture(name) {
        let actual = String::from_utf8_lossy(bytes);
        assert_eq!(actual, expected, "Capture {name} mismatch");
    } else {
        panic!("Capture {name} not found");
    }
}

pub fn print_tokenized(s: &str) {
    let trie = PARSER_FACTORY.tok_env().tok_trie();
    let tokens = PARSER_FACTORY.tok_env().tokenize(s);
    println!("{:?}", trie.test_trace_tokens(&tokens));
}

// ── String / schema validation helpers ───────────────────────────────────────

#[derive(Debug)]
pub enum NumericBounds {
    Inclusive,
    Exclusive,
}

pub fn make_parser(lark: &str, quiet: bool) -> Result<TokenParser> {
    let grm = TopLevelGrammar::from_lark(lark.to_string());
    let mut parser = get_parser_factory().create_parser_from_init(
        GrammarInit::Serialized(grm),
        if quiet { 0 } else { 2 },
        if quiet { 1 } else { 2 },
    )?;
    parser.start_without_prompt();
    Ok(parser)
}

pub fn consume(parser: &mut TokenParser, tok: u32) {
    let n = parser.consume_token(tok).unwrap();
    assert!(n == 0);
}

pub fn lark_ok(lark: &str) {
    if let Err(e) = make_parser(lark, false) {
        panic!("unexpected error: {e}, grm:\n{lark}")
    }
}

pub fn lark_err_test(lark: &str, err: &str) {
    match make_parser(lark, false) {
        Err(e) => {
            let e = format!("{e}");
            if !e.contains(err) {
                panic!("unexpected error: {e}, expecting {err:?}; grm:\n{lark}");
            }
        }
        Ok(_) => panic!("expected error: {err}; grm:\n{lark}"),
    }
}

pub fn json_err_test(schema: &Value, err: &str) {
    lark_err_test(
        &format!(r#"start: %json {}"#, serde_json::to_string(schema).unwrap()),
        err,
    );
}

pub fn lark_str_test(lark: &str, should_accept: bool, input: &str, quiet: bool) {
    let trie = get_tok_env().tok_trie();
    let (final_reject, input) = if let Some(input) = input.strip_prefix("FINAL_REJECT:") {
        (true, input)
    } else {
        (false, input)
    };
    let tokens = get_tok_env().tokenize(input);
    let info = format!(
        "\ninput: {:?}, grm: {:?}",
        limit_str(input, 500),
        limit_str(lark, 100)
    );
    if !quiet {
        println!(
            "\n\ntokens: {}, accpt={}\ngrm:\n{}\n",
            trie.tokens_dbg(&tokens),
            should_accept,
            lark
        );
    }

    // let t0 = std::time::Instant::now();
    let mut p = make_parser(lark, quiet).unwrap();
    // println!("make_parser: {:?}", t0.elapsed());

    for tok in tokens.iter() {
        let m = p.compute_mask().unwrap();
        if m.is_allowed(*tok) {
            consume(&mut p, *tok);
        } else {
            if should_accept {
                panic!("unexpected token: {}{info}", trie.token_dbg(*tok));
            }
            if final_reject {
                panic!(
                    "unexpected token: {}; expecting reject only at the end{info}",
                    trie.token_dbg(*tok)
                );
            }
            return;
        }
    }

    if !final_reject && !should_accept {
        panic!(
            "expected rejection (in the middle; final accept={})",
            p.is_accepting()
        );
    }

    if p.is_accepting() == final_reject {
        if p.is_accepting() {
            panic!("unexpected accept{info}");
        } else {
            panic!("unexpected reject{info}");
        }
    }
}

pub fn lark_str_test_many_ext(quiet: bool, lark: &str, passing: &[&str], failing: &[&str]) {
    for s in passing {
        lark_str_test(lark, true, s, quiet);
    }
    for s in failing {
        lark_str_test(lark, false, s, quiet);
    }
}

pub fn json_schema_check(schema: &Value, json_obj: &Value, expect_valid: bool) {
    /*
       This is a modification of the lark_str_test function, which makes the
       assumption that the input Value completely satifies the schema.

       The subtlety is that a string of tokens might not match a grammar _yet_
       but could with the addition of more tokens. For example, if we're trying
       to construct an integer which is greater than 2, then the string "1"
       is not yet a match, but it could become one if we add more tokens.
       lark_str_test uses the magic 'FINAL_REJECT:' rule to handle this,
       but we can write something a little simpler here.
    */
    let lark_grammar = format!(r#"start: %json {}"#, serde_json::to_string(schema).unwrap());
    let json_string = serde_json::to_string(json_obj).unwrap();

    // Tokenize the string representation of the JSON object
    let tokens = get_tok_env().tokenize(&json_string);

    // Create the parser
    let mut p = make_parser(&lark_grammar, false).unwrap();

    // Work through token by token
    for (i, tok) in tokens.iter().enumerate() {
        // Compute the mask of allowed tokens
        let m = p.compute_mask().unwrap();

        if m.is_allowed(*tok) {
            // Consume the token
            consume(&mut p, *tok);
        } else {
            // Token isn't allowed, so check if we expect this
            let curr_tok_str = get_tok_env().tok_trie().token_dbg(*tok);
            assert!(
                !expect_valid,
                "Unexpected token: {curr_tok_str} at token index {i}",
            );
            // We were expecting this to fail, so return early
            return;
        }
    }

    /*
    Note that p.is_accepting() will be true if the parser has reached a valid end state.
    It does not mean that we couldn't add more tokens and remain valid.
    For example, if we have a schema for any integer, then we can always add more digits
    to a valid integer string.
     */
    assert_eq!(p.is_accepting(), expect_valid, "Final state mismatch");
}

pub fn json_test_many(schema: &Value, passing: &[Value], failing: &[Value]) {
    for s in passing {
        json_schema_check(schema, s, true);
    }
    for s in failing {
        json_schema_check(schema, s, false);
    }
}

pub fn lark_str_test_many(lark: &str, passing: &[&str], failing: &[&str]) {
    lark_str_test_many_ext(false, lark, passing, failing);
}

pub fn lark_str_test_many_quiet(lark: &str, passing: &[&str], failing: &[&str]) {
    lark_str_test_many_ext(true, lark, passing, failing);
}
