#!/bin/sh

set -e

sed '/\/\/ ---START OF GENERATED CODE---/q' ../parser/src/cbison.rs > tmp.rs

bindgen \
    --allowlist-type 'cbison_factory' \
    --allowlist-item 'CBISON_.*' \
    --no-recursive-allowlist \
    cbison_api.h >> tmp.rs

mv tmp.rs ../parser/src/cbison.rs
