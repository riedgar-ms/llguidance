use toktrie::{TokRxInfo, TokTrie};

use crate::ParserFactory;

#[cxx::bridge]
mod ffi {
    unsafe extern "C++" {
        include!("rust/cxx.h");
        include!("llguidance_cxx_support.h");

        type TokenizerInit;

        fn vocab_size(self: &TokenizerInit) -> usize;
        fn tok_eos(self: &TokenizerInit) -> u32;
        fn token_bytes(self: &TokenizerInit, token: usize) -> &[u8];
    }
    extern "Rust" {
        type ParserFactory;

        fn parser_factory(tok_init: UniquePtr<TokenizerInit>) -> Box<ParserFactory>;
    }
}

fn parser_factory(tok_init: cxx::UniquePtr<ffi::TokenizerInit>) -> Box<ParserFactory> {
    let mut tokens = vec![];
    for tok in 0..tok_init.vocab_size() {
        tokens.push(tok_init.token_bytes(tok).to_vec());
    }
    let trie = TokTrie::from(
        &TokRxInfo {
            tok_eos: tok_init.tok_eos(),
            vocab_size: tokens.len() as u32,
            tok_bos: None,
            tok_pad: None,
            tok_unk: None,
            tok_end_of_turn: None,
        },
        &tokens,
    );
    let _ = trie;

    todo!()
}
