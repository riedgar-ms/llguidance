#!/bin/sh

bindgen \
    --allowlist-item 'cbison_.*' \
    cbison_api.h > cbison.rs

