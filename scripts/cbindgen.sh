#!/bin/sh

CHECK=0
if [ "$1" = "--check" ]; then
    CHECK=1
    shift
fi

if cbindgen --version ; then
    echo "cbindgen is already installed"
else
    echo "Installing cbindgen"
    cargo install cbindgen
fi


function generate() {
    local crate=$1

mkdir -p tmp
cbindgen --config ./cbindgen.toml \
         --crate "$crate" \
         --output tmp/llguidance0.h  > tmp/cbindgen.txt 2>&1

if [ $? -ne 0 ]; then
    echo "Failed to generate ${crate}.h"
    cat tmp/cbindgen.txt
    exit 1
else
    # print warnings and errors, but skip "Skip" messages
    grep -vE "(Skip .*\(not |WARN: Skipping llguidance_cbison::cbison_)" tmp/cbindgen.txt

    cat tmp/llguidance0.h | \
        sed -e 's@const LlgCbisonFactory \*@cbison_factory_t @g' \
            -e 's@const LlgCbisonTokenizer \*@cbison_tokenizer_t @g' \
            -e 's@CbisonTokenizer \*@cbison_tokenizer_t @g' \
        | \
        grep -v "\* # Safety" | \
        grep -v "\* This function should only be called from C code" \
    > tmp/llguidance.h

    if diff -u ${crate}.h tmp/llguidance.h; then
        echo "${crate}.h is up to date"
    else
        if [ $CHECK -eq 1 ]; then
            echo "${crate}.h is out of date"
            exit 1
        else
            cp tmp/llguidance.h ${crate}.h
            echo "Updated ${crate}.h"
        fi
    fi
fi
}

cd "$(dirname "$0")/../parser"
generate llguidance
cd ../llguidance_cbison
generate llguidance_cbison
