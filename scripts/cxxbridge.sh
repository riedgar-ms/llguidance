#!/bin/sh

CHECK=0
if [ "$1" = "--check" ]; then
    CHECK=1
    shift
fi

if cxxbridge --version ; then
    echo "cxxbridge is already installed"
else
    echo "Installing cxxbridge-cmd"
    cargo install cxxbridge-cmd
fi

cd "$(dirname "$0")/../parser"

mkdir -p tmp
set -e
cxxbridge --header > tmp/cxx.h
cxxbridge src/cxx_ffi.rs -o tmp/llguidance_cxx.h
cxxbridge src/cxx_ffi.rs -o tmp/llguidance_cxx.cc
set +e

for f in llguidance_cxx.h llguidance_cxx.cc cxx.h; do
    sed -i -e 's@rust/cxx.h@cxx.h@' tmp/$f
    if diff -u cxx/$f tmp/$f; then
        echo "cxx/$f is up to date"
    else
        if [ $CHECK -eq 1 ]; then
            echo "cxx/$f is out of date"
            exit 1
        else
            cp tmp/$f cxx/$f
            echo "Updated cxx/$f"
        fi
    fi
done

cd cxx
c++ -std=c++20 -c llguidance_cxx.cc
