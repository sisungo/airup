#!/bin/sh

function compile() {
    ${CC} ${CFLAGS} -c ./src/$1.c -o ./target/$1.o
}
function sdk_dylib_filename() {
    if [ $(uname) = "Darwin" ]; then
        echo libairup_sdk.dylib
    else
        echo libairup_sdk.so
    fi
}
function link_all() {
    ${CC} -shared -fPIC ./target/*.o -o ./target/$(sdk_dylib_filename)
}

rm -rf ./target
mkdir -p ./target

CC=${CC:-cc}
CFLAGS="${CFLAGS} -std=c11 -I./include -Wall -Wextra -Wpedantic"

cd $(dirname $0)

compile rpc
compile lib

link_all
