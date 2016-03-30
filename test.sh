#!/bin/sh

preload () {
    local library
    library=$1
    shift
    if [ "$(uname)" = "Darwin" ]; then
        DYLD_INSERT_LIBRARIES=target/debug/"$library".dylib "$@"
    else
        LD_PRELOAD=target/debug/"$library".so "$@"
    fi
}

set -ex

cd examples/readlinkspy
cargo update
cargo build
preload libreadlinkspy ls -l /dev/stdin | grep readlink
