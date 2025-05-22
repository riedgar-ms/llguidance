use std::sync::Arc;

use anyhow::{ensure, Result};
use llguidance::toktrie::{self, TokEnv, TokRxInfo, TokTrie, TokenizerEnv};

type LlamaTokenizeFn = unsafe extern "C" fn(
    vocab: *const std::os::raw::c_void,
    text: *const std::os::raw::c_char,
    text_len: i32,
    tokens: *mut i32,
    n_tokens_max: i32,
    add_special: bool,
    parse_special: bool,
) -> i32;

struct LlamaTokenizerInner {
    trie: TokTrie,
    tokenize_fn: LlamaTokenizeFn,
    vocab: *const std::os::raw::c_void,
}
// SAFETY: tokenize_fn is required to be thread-safe
unsafe impl Send for LlamaTokenizerInner {}
unsafe impl Sync for LlamaTokenizerInner {}

impl LlamaTokenizerInner {
    fn raw_tokenize(&self, s: &[u8]) -> Vec<toktrie::TokenId> {
        let mut res_toks = vec![0u32; s.len() / 4 + 5];
        let res = unsafe {
            (self.tokenize_fn)(
                self.vocab,
                s.as_ptr() as *const std::os::raw::c_char,
                s.len().try_into().unwrap(),
                res_toks.as_mut_ptr() as *mut i32,
                res_toks.len().try_into().unwrap(),
                false,
                false,
            )
        };

        let res = if res < 0 {
            let n_toks = (-res) as usize;
            res_toks.resize(n_toks, 0);
            let res2 = unsafe {
                (self.tokenize_fn)(
                    self.vocab,
                    s.as_ptr() as *const std::os::raw::c_char,
                    s.len().try_into().unwrap(),
                    res_toks.as_mut_ptr() as *mut i32,
                    res_toks.len().try_into().unwrap(),
                    false,
                    false,
                )
            };
            assert!(res2 == n_toks as i32);
            res2
        } else {
            res
        };

        res_toks.truncate(res as usize);
        res_toks
    }
}

impl TokenizerEnv for LlamaTokenizerInner {
    fn tok_trie(&self) -> &TokTrie {
        &self.trie
    }

    fn tokenize_bytes(&self, s: &[u8]) -> Vec<toktrie::TokenId> {
        // llama.cpp tokenizer encodes invalid UTF8 as Unicode replacement character U+FFFD,
        // so we need the greedy fallback
        self.trie
            .tokenize_with_greedy_fallback(s, |s| self.raw_tokenize(s.as_bytes()))
    }
}

pub fn tokenv_from_llamacpp(
    tokens: Vec<Vec<u8>>,
    vocab_ptr: usize,
    tokenize_fptr: usize,
    eos_token: u32,
) -> Result<TokEnv> {
    ensure!(vocab_ptr != 0, "vocab_ptr must be non-null");
    ensure!(tokenize_fptr != 0, "tokenize_fptr must be non-null");

    let info = TokRxInfo::new(tokens.len() as u32, eos_token);
    let trie = TokTrie::from(&info, &tokens);

    let llama_tok = LlamaTokenizerInner {
        trie,
        tokenize_fn: unsafe { std::mem::transmute::<usize, LlamaTokenizeFn>(tokenize_fptr) },
        vocab: vocab_ptr as *const std::os::raw::c_void,
    };
    Ok(Arc::new(llama_tok))
}
