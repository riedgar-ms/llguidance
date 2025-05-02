#!/bin/sh

set -e

sed '/\/\/ ---START OF GENERATED CODE---/q' ../llguidance_cbison/src/cbison.rs > tmp.rs

bindgen \
    --allowlist-type 'cbison_factory' \
    --allowlist-type 'cbison_tokenizer' \
    --allowlist-type 'cbison_mask_req.*' \
    --allowlist-item 'CBISON_.*' \
    --no-recursive-allowlist \
    cbison_api.h >> tmp.rs

mv tmp.rs ../llguidance_cbison/src/cbison.rs
clang2py cbison_api.h | sed \
    -e 's@ctypes.c_uint64@ctypes.c_size_t@' \
    -e 's@, ctypes.POINTER(struct_cbison_factory)@, cbison_factory_t@' \
    -e 's@, ctypes.POINTER(struct_cbison_matcher)@, cbison_matcher_t@' \
    -e 's@(ctypes.POINTER(struct_cbison_matcher)@(cbison_matcher_t@' \
    -e 's@ctypes.POINTER(ctypes.c_char)@ctypes.c_char_p@g' \
    > cbison/bindings.py