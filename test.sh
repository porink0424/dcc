#!/bin/bash
echo "test started."
echo

assert() {
    expected="$1"
    input="$2"

    ./target/debug/dcc "$input" > tmp.s
    # アセンブルと実行をdocker環境でやらせる
    docker run --rm -v $(cd $(dirname $0) && pwd):/dcc -w /dcc dcc /bin/sh -c "cc -o tmp tmp.s; ./tmp"

    actual="$?"

    if [ "$actual" = "$expected" ]; then
        echo "\`\`\`$input\`\`\`"
        echo "=> $actual"
        echo
    else
        echo "\`\`\`$input\`\`\`"
        echo "=> $expected expected, but got $actual"
        echo
        exit 1
    fi
}

# assert 0 '0;'
# assert 42 '42;'
# assert 41 " 12 + 34 -   5 ;"
# assert 47 '5+6*7;'
# assert 15 '5*(9-6);'
# assert 4 '(3+5)/2;'
# assert 10 '-10+20;'
# assert 1 '-5--6;' 
# assert 19 '-5-+6+30;'
# assert 1 '(4 == 2) + (3 < 5);'
# assert 1 '6 >= 2;'
# assert 1 '6 != 2;'
assert 4 '
a = 4;
a;
'
assert 14 '
a = 3;
b = 5 * 6 - 8;
a + b / 2;
'

echo "test finished successfully."