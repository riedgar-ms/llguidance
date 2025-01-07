use anyhow::{bail, Result};
use llguidance::{
    api::{ParserLimits, TopLevelGrammar},
    toktrie::{InferenceCapabilities, TokEnv},
    Constraint, JsonCompileOptions, TokenParser,
};
use serde_json::Value;

use crate::cases::Test;

use super::{
    cases::Case,
    results::{CaseResult, TestResult},
};

pub fn run_case_tests(case: Case, tok_env: TokEnv) -> Vec<TestResult> {
    let opts = JsonCompileOptions::default();
    let compiled = opts.json_to_llg_no_validate(case.schema);
    if compiled.is_err() {
        // If the tests are all invalid, let's call a compile error a success. I guess.
        if case.tests.iter().all(|test| !test.valid) {
            return case
                .tests
                .iter()
                .map(|_| TestResult {
                    valid: false,
                    success: true,
                })
                .collect::<Vec<_>>();
        }
        return vec![TestResult {
            valid: false,
            success: false,
        }];
    }
    let grammar = compiled.unwrap();
    let parser = TokenParser::from_llguidance_json(
        tok_env,
        grammar,
        llguidance::Logger::new(0, 1),
        InferenceCapabilities {
            ff_tokens: true,
            backtrack: false,
            conditional_ff_tokens: false,
            fork: false,
        },
        ParserLimits::default(),
        vec![],
    )
    .unwrap();
    let constraint = Constraint::new(parser);
    case.tests
        .into_iter()
        .map(|test| {
            let inner_result = run_test(test.data, constraint.deep_clone());
            TestResult {
                valid: test.valid,
                success: inner_result.is_ok() == test.valid,
            }
        })
        .collect()
}

fn run_test(data: Value, mut constraint: Constraint) -> Result<()> {
    let data_str = data.to_string();
    let tokens = constraint.parser.token_env.tokenize(&data_str);

    let mut idx = 0;
    while idx < tokens.len() {
        let res = constraint.compute_mask()?;
        if res.is_stop() {
            bail!("premature stop");
        }

        let sampled_token = if let Some(mask) = &res.sample_mask {
            let sampled_token = tokens[idx];
            if !mask.is_allowed(sampled_token) {
                bail!("sampled token not allowed by mask");
            }
            Some(sampled_token)
        } else {
            None
        };

        let splice = constraint.commit_token(sampled_token)?;
        if splice.stop {
            if idx + 1 < tokens.len() {
                bail!("premature stop (commit)");
            } else {
                return Ok(());
            }
        }

        assert!(splice.backtrack == 0); // we didn't allow backtracking in InferenceCaps

        if safe_slice(&tokens, idx, idx + splice.ff_tokens.len()) != splice.ff_tokens {
            bail!("ff_tokens mismatch");
        }

        idx += splice.ff_tokens.len();
    }
    let accept = constraint.parser.is_accepting();
    if !accept {
        bail!("parser did not accept");
    } else {
        Ok(())
    }
}

fn safe_slice<T>(vec: &[T], start: usize, end: usize) -> &[T] {
    let end = end.min(vec.len());
    &vec[start..end]
}
