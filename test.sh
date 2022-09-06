#!/bin/bash
assert() {
    expected="$1"
    input="$2"

    ./target/debug/dcc "$input" > tmp.s
    docker run --rm -v $(cd $(dirname $0) && pwd):/dcc -w /dcc dcc cc -o tmp tmp.s
    docker run --rm -v $(cd $(dirname $0) && pwd):/dcc -w /dcc dcc ./tmp

    actual="$?"

    if [ "$actual" = "$expected" ]; then
        echo "$input => $actual"
    else
        echo "$input => $expected expected, but got $actual"
        exit 1
    fi
}

assert 0 0
assert 42 42

echo OK