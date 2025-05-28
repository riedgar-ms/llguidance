use std::{
    ffi::c_char,
    sync::{atomic::AtomicUsize, Arc},
};

use anyhow::{anyhow, bail, Result};
use serde::{Deserialize, Serialize};

use crate::cbison::{
    cbison_mask_req, cbison_tokenizer_ptr_t, cbison_tokenizer_t, CbisonFactory, CbisonTokenizer,
    CBISON_FACTORY_MAGIC, CBISON_FACTORY_VERSION_MAJOR, CBISON_FACTORY_VERSION_MINOR,
    CBISON_TOKENIZER_MAGIC, CBISON_TOKENIZER_VERSION_MAJOR, CBISON_TOKENIZER_VERSION_MINOR,
};

use llguidance::{
    api::ParserLimits,
    earley::SlicedBiasComputer,
    ffi::*,
    toktrie::{
        ApproximateTokEnv, InferenceCapabilities, TokEnv, TokRxInfo, TokTrie, TokenId, TokenizerEnv,
    },
    ParserFactory,
};

#[repr(C)]
pub struct LlgCbisonFactory {
    common: CbisonFactory, // has to come first!
    ref_count: AtomicUsize,
    tokenizer: LlgTokenizer,
    executor: LlgExecutor,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct JsonFactoryOptions {
    pub slices: Option<Vec<String>>,
    #[serde(default)]
    pub limits: ParserLimits,
    #[serde(default)]
    pub num_threads: u32,
    // default is 1
    pub stderr_log_level: Option<u32>,
}

impl LlgCbisonFactory {
    fn from_factory_init(init: &LlgFactoryInit) -> Result<LlgCbisonFactory> {
        let tok = LlgTokenizer::from_factory_init(init)?;
        let factory = LlgCbisonFactory {
            common: fill_cbison_factory(tok.tok_env()),
            ref_count: AtomicUsize::new(1),
            tokenizer: tok,
            executor: LlgExecutor::new(&init.executor)?,
        };
        Ok(factory)
    }

    unsafe fn from_tokenizer_and_options(
        tokenizer: *mut CbisonTokenizer,
        options_json: *const c_char,
    ) -> Result<LlgCbisonFactory> {
        let tok_env: TokEnv = Arc::new(CbisonTokEnv::new(tokenizer)?);
        let options_json = c_str_to_json(options_json, "options_json")?;
        let options: JsonFactoryOptions = serde_json::from_str(options_json)
            .map_err(|e| anyhow::anyhow!("Invalid JSON in options: {e}"))?;
        let slices = options
            .slices
            .unwrap_or_else(SlicedBiasComputer::general_slices);
        let mut factory = ParserFactory::new(&tok_env, InferenceCapabilities::default(), &slices)?;
        *factory.limits_mut() = options.limits;
        if let Some(log_level) = options.stderr_log_level {
            factory.set_stderr_log_level(log_level);
        }

        Self::from_parser_factory(
            Arc::new(factory),
            LlgExecutor::new(&LlgExecutorInit {
                num_threads: options.num_threads,
            })?,
        )
    }

    pub fn from_parser_factory(
        parser_factory: Arc<ParserFactory>,
        executor: LlgExecutor,
    ) -> Result<LlgCbisonFactory> {
        Ok(LlgCbisonFactory {
            common: fill_cbison_factory(parser_factory.tok_env()),
            ref_count: AtomicUsize::new(1),
            tokenizer: LlgTokenizer::from_factory(parser_factory),
            executor,
        })
    }

    fn factory(&self) -> &ParserFactory {
        self.tokenizer.factory()
    }

    fn check_magic(&self) {
        assert!(
            self.common.magic == CBISON_FACTORY_MAGIC
                && self.common.impl_magic == CBISON_IMPL_MAGIC
        );
    }

    fn constraint_init(&self) -> LlgConstraintInit {
        self.check_magic();
        let f = self.factory();
        LlgConstraintInit {
            tokenizer: &self.tokenizer,
            log_buffer_level: f.buffer_log_level(),
            log_stderr_level: f.stderr_log_level(),
            ff_tokens_ok: false,
            backtrack_ok: false,
            limits: f.limits().clone(),
        }
    }
}

const CBISON_IMPL_MAGIC: u32 = 0x4C4C4742; // "LLG\0"

unsafe extern "C" fn cbison_validate_grammar(
    this: &LlgCbisonFactory,
    grammar_type: *const c_char,
    grammar: *const c_char,
    message: *mut c_char,
    message_len: usize,
) -> i32 {
    let init = this.constraint_init();
    llg_validate_grammar(&init, grammar_type, grammar, message, message_len)
}

unsafe extern "C" fn cbison_new_matcher(
    this: &LlgCbisonFactory,
    grammar_type: *const c_char,
    grammar: *const c_char,
) -> *mut LlgMatcher {
    let init = this.constraint_init();
    llg_new_matcher(&init, grammar_type, grammar)
}

unsafe extern "C" fn cbison_matcher_is_stopped(matcher: &mut LlgMatcher) -> bool {
    llg_matcher_is_stopped(matcher)
}

unsafe extern "C" fn cbison_clone_matcher(matcher: &mut LlgMatcher) -> *mut LlgMatcher {
    llg_clone_matcher(matcher)
}

unsafe extern "C" fn cbison_factory_incr_ref_count(factory: &LlgCbisonFactory) {
    factory.check_magic();
    factory.ref_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
}

unsafe extern "C" fn cbison_factory_decr_ref_count(factory: &LlgCbisonFactory) {
    factory.check_magic();
    let count = factory
        .ref_count
        .fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
    if count == 1 {
        // Last reference, we can drop the factory
        let factory =
            unsafe { Box::from_raw(factory as *const LlgCbisonFactory as *mut LlgCbisonFactory) };
        drop(factory);
    }
}

fn fill_cbison_factory(tok_env: &TokEnv) -> CbisonFactory {
    let trie = tok_env.tok_trie();
    CbisonFactory {
        magic: CBISON_FACTORY_MAGIC,
        impl_magic: CBISON_IMPL_MAGIC,
        version_major: CBISON_FACTORY_VERSION_MAJOR,
        version_minor: CBISON_FACTORY_VERSION_MINOR,
        n_vocab: trie.vocab_size(),
        eos_token_id: trie.eos_token(),
        reserved_hd: [0; 7],
        impl_data: std::ptr::null_mut(),
        mask_byte_len: trie.vocab_size().div_ceil(32) * 4,
        incr_ref_count: Some(cbison_factory_incr_ref_count),
        decr_ref_count: Some(cbison_factory_decr_ref_count),
        validate_grammar: Some(cbison_validate_grammar),
        new_matcher: Some(cbison_new_matcher),
        get_error: Some(llg_matcher_get_error),
        compute_mask: Some(llg_matcher_compute_mask_into),
        consume_tokens: Some(llg_matcher_consume_tokens),
        is_accepting: Some(llg_matcher_is_accepting),
        is_stopped: Some(cbison_matcher_is_stopped),
        validate_tokens: Some(llg_matcher_validate_tokens),
        compute_ff_tokens: Some(llg_matcher_compute_ff_tokens),
        free_matcher: Some(llg_free_matcher),
        rollback: Some(llg_matcher_rollback),
        reset: Some(llg_matcher_reset),
        clone_matcher: Some(cbison_clone_matcher),
        #[cfg(not(feature = "rayon"))]
        compute_masks: None,
        #[cfg(feature = "rayon")]
        compute_masks: Some(cbison_compute_masks),
        reserved_ptr: [std::ptr::null_mut(); 16],
    }
}

unsafe impl Send for cbison_mask_req {}
impl Clone for cbison_mask_req {
    fn clone(&self) -> Self {
        cbison_mask_req {
            matcher: self.matcher,
            mask_dest: self.mask_dest,
        }
    }
}

#[cfg(feature = "rayon")]
unsafe extern "C" fn cbison_compute_masks(
    this: &LlgCbisonFactory,
    reqs: *mut cbison_mask_req,
    n_reqs: usize,
) -> i32 {
    if n_reqs == 0 {
        return 0;
    }
    if reqs.is_null() {
        return -1;
    }
    let reqs = unsafe { slice_from_ptr(reqs, n_reqs).unwrap() };
    let byte_len = this.common.mask_byte_len;

    this.executor.for_each(reqs.to_vec(), |req| {
        llg_matcher_compute_mask_into(&mut *req.matcher, req.mask_dest, byte_len);
    });

    0
}

/// Construct a new cbison factory for a given tokenizer.
/// # Safety
/// This function should only be called from C code.
#[no_mangle]
pub unsafe extern "C" fn llg_cbison_new_factory_init(
    init: &LlgFactoryInit,
    error_string: *mut c_char,
    error_string_len: usize,
) -> *const LlgCbisonFactory {
    match LlgCbisonFactory::from_factory_init(init) {
        Ok(factory) => Box::into_raw(Box::new(factory)),
        Err(e) => {
            save_error_string(e, error_string, error_string_len);
            std::ptr::null_mut()
        }
    }
}

/// Construct a new CBISON factory for a given tokenizer and options.
/// The reference count of the tokenizer is incremented (until the factory is freed).
/// `options_json` is an optional JSON string with the following (optional) fields:
/// - `slices`: a list of slice names (if not provided, the default slices will be used).
/// - `limits`: a JSON object with the parser limits (if not provided, the default limits will be used).
/// - `num_threads`: the number of threads to use (if not provided, the default is 80% of cores up to 32).
/// - `stderr_log_level`: the log level for stderr (if not provided, the default is 1).
/// # Safety
/// This function should only be called from C code.
#[no_mangle]
pub unsafe extern "C" fn llg_cbison_new_factory(
    tokenizer: *mut CbisonTokenizer,
    options_json: *const c_char,
    error_string: *mut c_char,
    error_string_len: usize,
) -> *const LlgCbisonFactory {
    match LlgCbisonFactory::from_tokenizer_and_options(tokenizer, options_json) {
        Ok(factory) => Box::into_raw(Box::new(factory)),
        Err(e) => {
            save_error_string(e, error_string, error_string_len);
            std::ptr::null_mut()
        }
    }
}

#[derive(Clone)]
#[repr(C)]
pub struct LlgCbisonTokenizer {
    common: CbisonTokenizer, // has to come first!
    ref_count: usize,
    tok_env: TokEnv,
}

unsafe extern "C" fn cbison_get_token(
    api: cbison_tokenizer_t,
    token_id: u32,
    bytes: *mut u8,
    bytes_len: usize,
) -> i32 {
    let api = LlgCbisonTokenizer::from_ptr(api);
    if token_id as usize >= api.common.n_vocab {
        return -1;
    }

    let t = api.tok_env.tok_trie().token(token_id);
    let t = if !t.is_empty() && t[0] == TokTrie::SPECIAL_TOKEN_MARKER {
        &t[1..]
    } else {
        t
    };

    if !bytes.is_null() && bytes_len > 0 {
        let len = std::cmp::min(t.len(), bytes_len);
        // SAFETY: t is from toktrie thus non-overlapping, bytes is non-null
        unsafe {
            std::ptr::copy_nonoverlapping(t.as_ptr(), bytes, len);
        }
    }

    t.len() as i32
}

extern "C" fn cbison_is_special_token(api: cbison_tokenizer_t, token_id: u32) -> i32 {
    let api = LlgCbisonTokenizer::from_ptr(api);
    if token_id as usize >= api.common.n_vocab {
        return -1;
    }

    let trie = api.tok_env.tok_trie();
    if trie.is_special_token(token_id) {
        1
    } else {
        0
    }
}

unsafe extern "C" fn cbison_tokenize_bytes(
    api: cbison_tokenizer_t,
    bytes: *const c_char,
    bytes_len: usize,
    output_tokens: *mut u32,
    output_tokens_len: usize,
) -> usize {
    let api = LlgCbisonTokenizer::from_ptr(api);
    let tokens = api
        .tok_env
        .tokenize_bytes(unsafe { slice_from_ptr_or_empty(bytes as *const u8, bytes_len) });
    let n_toks = tokens.len();
    if output_tokens.is_null() {
        return n_toks;
    }
    let to_copy = std::cmp::min(n_toks, output_tokens_len);
    // SAFETY: tokens is freshly allocated and thus non-overlapping, output_tokens is non-null
    unsafe {
        std::ptr::copy_nonoverlapping(tokens.as_ptr(), output_tokens, to_copy);
    }
    n_toks
}

unsafe extern "C" fn cbison_decr(api: cbison_tokenizer_ptr_t) {
    assert!(!api.is_null());
    let api = &mut *(api as *mut LlgCbisonTokenizer);
    api.check_magic();
    api.ref_count -= 1;
    if api.ref_count == 0 {
        drop(Box::from_raw(api));
    }
}

unsafe extern "C" fn cbison_incr(api: cbison_tokenizer_ptr_t) {
    assert!(!api.is_null());
    let api = &mut *(api as *mut LlgCbisonTokenizer);
    api.check_magic();
    api.ref_count += 1;
}

impl LlgCbisonTokenizer {
    pub fn new(tok_env: TokEnv) -> Self {
        let trie = tok_env.tok_trie();
        let n_vocab = trie.vocab_size();
        let eos_token_id = trie.eos_token();
        LlgCbisonTokenizer {
            common: CbisonTokenizer {
                magic: CBISON_TOKENIZER_MAGIC,
                impl_magic: CBISON_IMPL_MAGIC,
                impl_data: std::ptr::null_mut(),
                version_major: CBISON_TOKENIZER_VERSION_MAJOR,
                version_minor: CBISON_TOKENIZER_VERSION_MINOR,
                n_vocab,
                eos_token_id,
                tokenize_bytes_requires_utf8: false,
                reserved_hd: [0; 6],
                get_token: Some(cbison_get_token),
                is_special_token: Some(cbison_is_special_token),
                tokenize_bytes: Some(cbison_tokenize_bytes),
                decr_ref_count: Some(cbison_decr),
                incr_ref_count: Some(cbison_incr),
                reserved_ptr: [std::ptr::null_mut(); 16],
            },
            ref_count: 1,
            tok_env,
        }
    }

    pub fn from_ptr(p: *const CbisonTokenizer) -> &'static Self {
        assert!(!p.is_null());
        let api = unsafe { &*(p as *const LlgCbisonTokenizer) };
        api.check_magic();
        api
    }

    fn check_magic(&self) {
        assert!(self.common.magic == CBISON_TOKENIZER_MAGIC);
        assert!(self.common.impl_magic == CBISON_IMPL_MAGIC);
        assert!(self.ref_count > 0);
    }
}

/// This for testing purposes only.
#[no_mangle]
pub extern "C" fn llg_cbison_new_byte_tokenizer() -> *const LlgCbisonTokenizer {
    let tok = LlgCbisonTokenizer::new(ApproximateTokEnv::single_byte_env());
    Box::into_raw(Box::new(tok))
}

struct CbisonTokEnv {
    cbison_tokenizer: *const CbisonTokenizer,
    trie: TokTrie,
}

unsafe impl Send for CbisonTokEnv {}
unsafe impl Sync for CbisonTokEnv {}

impl Drop for CbisonTokEnv {
    fn drop(&mut self) {
        assert!(!self.cbison_tokenizer.is_null());
        let decr =
            unsafe { (*self.cbison_tokenizer).decr_ref_count }.expect("decr_ref_count is null");
        unsafe { decr(self.cbison_tokenizer as *mut _) }
    }
}

impl CbisonTokEnv {
    fn new(cbison_tokenizer: *mut CbisonTokenizer) -> Result<Self> {
        let tok = unsafe { &*cbison_tokenizer };

        let incr = tok
            .incr_ref_count
            .ok_or_else(|| anyhow!("cbison_tokenizer does not have incr_ref_count function"))?;
        unsafe { incr(cbison_tokenizer) };

        let get_token = tok
            .get_token
            .ok_or_else(|| anyhow!("cbison_tokenizer does not have get_token function"))?;
        let is_special_token = tok
            .is_special_token
            .ok_or_else(|| anyhow!("cbison_tokenizer does not have is_special_token function"))?;
        let mut buf = vec![0u8; 1024];
        let mut tokens = Vec::with_capacity(tok.n_vocab);
        for tok_id in 0..(tok.n_vocab as u32) {
            let len = unsafe { get_token(cbison_tokenizer, tok_id, buf.as_mut_ptr(), buf.len()) };
            if len < 0 {
                return Err(anyhow!("get_token failed for token {tok_id}"));
            }
            if len > (buf.len() as i32) - 2 {
                return Err(anyhow!(
                    "get_token returned too many bytes for token {tok_id}"
                ));
            }

            let mut bytes = buf[0..len as usize].to_vec();

            let is_special = unsafe { is_special_token(cbison_tokenizer, tok_id) } != 0;
            if is_special {
                bytes.insert(0, TokTrie::SPECIAL_TOKEN_MARKER);
            }

            tokens.push(bytes);
        }

        let info = TokRxInfo {
            vocab_size: tok.n_vocab.try_into().unwrap(),
            tok_eos: tok.eos_token_id,
            tok_bos: None,
            tok_pad: None,
            tok_unk: None,
            tok_end_of_turn: None,
        };

        let trie = TokTrie::from(&info, &tokens);

        Ok(CbisonTokEnv {
            cbison_tokenizer,
            trie,
        })
    }

    fn raw_tokenize(&self, s: &[u8]) -> Vec<TokenId> {
        let tok_fn = unsafe { (*self.cbison_tokenizer).tokenize_bytes };
        if let Some(tokenize_fn) = tok_fn {
            let mut res_toks = vec![0; s.len() / 4 + 5];
            let n_toks = unsafe {
                tokenize_fn(
                    self.cbison_tokenizer,
                    s.as_ptr() as *const c_char,
                    s.len(),
                    res_toks.as_mut_ptr(),
                    res_toks.len(),
                )
            };

            if n_toks > res_toks.len() {
                res_toks.resize(n_toks, 0);
                unsafe {
                    tokenize_fn(
                        self.cbison_tokenizer,
                        s.as_ptr() as *const c_char,
                        s.len(),
                        res_toks.as_mut_ptr(),
                        res_toks.len(),
                    )
                };
            }

            res_toks.truncate(n_toks);
            res_toks
        } else {
            self.trie.greedy_tokenize(s)
        }
    }

    fn has_tokenize_fn(&self) -> bool {
        let tok_fn = unsafe { (*self.cbison_tokenizer).tokenize_bytes };
        tok_fn.is_some()
    }
}

impl TokenizerEnv for CbisonTokEnv {
    fn tok_trie(&self) -> &TokTrie {
        &self.trie
    }

    fn tokenize_is_canonical(&self) -> bool {
        self.has_tokenize_fn()
    }

    fn tokenize_bytes(&self, s: &[u8]) -> Vec<TokenId> {
        if self.has_tokenize_fn() {
            let utf8_required = unsafe { (*self.cbison_tokenizer).tokenize_bytes_requires_utf8 };
            if utf8_required {
                self.trie
                    .tokenize_with_greedy_fallback(s, |s| self.raw_tokenize(s.as_bytes()))
            } else {
                self.raw_tokenize(s)
            }
        } else {
            self.trie.greedy_tokenize(s)
        }
    }
}

unsafe fn slice_from_ptr<'a, T>(data: *const T, len: usize) -> Result<&'a [T]> {
    if len == 0 {
        return Ok(&[]);
    }
    if data.is_null() {
        bail!("Null pointer");
    }
    Ok(std::slice::from_raw_parts(data, len))
}

unsafe fn slice_from_ptr_or_empty<'a, T>(data: *const T, len: usize) -> &'a [T] {
    if len == 0 || data.is_null() {
        &[]
    } else {
        std::slice::from_raw_parts(data, len)
    }
}

unsafe fn hf_tokenizer(
    tokenizer_json: *const c_char,
    options_json: *const c_char,
) -> Result<LlgCbisonTokenizer> {
    let options_json = c_str_to_json(options_json, "options_json")?;
    let options: LlgJsonTokenizerOptions = serde_json::from_str(options_json)
        .map_err(|e| anyhow::anyhow!("Invalid JSON in options: {e}"))?;
    let tokenizer_json = c_str_to_str(tokenizer_json, "tokenizer_json")?;
    let mut tokenizer =
        toktrie_hf_tokenizers::ByteTokenizer::from_json_bytes(tokenizer_json.as_bytes())?;
    if let Some(tok) = options.eos_token_id {
        tokenizer.set_eos_token(tok);
    }
    let env = tokenizer.into_tok_env(options.n_vocab)?;
    Ok(LlgCbisonTokenizer::new(env))
}

#[derive(Clone, Serialize, Deserialize)]
pub struct LlgJsonTokenizerOptions {
    pub n_vocab: Option<usize>,
    pub eos_token_id: Option<u32>,
}

/// Construct a new cbison tokenizer from a JSON string representing a HuggingFace
/// fast tokenizer (tokenizer.json file).
/// `options` is a an optional JSON string with the following (optional) fields:
/// - `n_vocab`: the vocabulary size (if not provided, it will be inferred from the tokenizer).
/// - `eos_token_id`: the end of sequence token id (if not provided, it will be inferred from the tokenizer).
/// # Safety
/// This function should only be called from C code.
#[no_mangle]
pub unsafe extern "C" fn llg_cbison_new_hf_tokenizer(
    tokenizer_json: *const c_char,
    options_json: *const c_char,
    error_string: *mut c_char,
    error_string_len: usize,
) -> *const LlgCbisonTokenizer {
    match hf_tokenizer(tokenizer_json, options_json) {
        Ok(tok) => Box::into_raw(Box::new(tok)),
        Err(e) => {
            save_error_string(e, error_string, error_string_len);
            std::ptr::null_mut()
        }
    }
}

unsafe fn c_str_to_json<'a>(c_str: *const c_char, info: &str) -> Result<&'a str> {
    if c_str.is_null() {
        Ok("{}")
    } else {
        c_str_to_str(c_str, info)
    }
}
