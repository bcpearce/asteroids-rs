#!/bin/bash
mkdir -p dist
echo "* binary" > $TRUNK_STAGING_DIR/.gitattributes

if [[ $INCLUDE_COVERAGE_REPORT ]]; then
    if [[ $TRUNK_PROFILE -eq "release" ]]; then
        cargo llvm-cov --release test --html --workspace
    else 
        cargo llvm-cov test --html --workspace
    fi
    mkdir -p $TRUNK_STAGING_DIR/code-cov
    cp target/llvm-cov/html/* $TRUNK_STAGING_DIR/code-cov/ -r
fi
