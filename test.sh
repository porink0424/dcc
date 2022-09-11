#!/bin/bash
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
PURPLE='\033[0;35m'
NC='\033[0m'

echo
echo -e "${PURPLE}test started.${NC}"
echo

assert() {
    expected="$1"
    input="$2"

    # dccによるコンパイル
    ./target/debug/dcc "$input" > tmp.s
    # アセンブルと実行をdocker環境でやらせる
    docker run --rm -v $(cd $(dirname $0) && pwd):/dcc -w /dcc dcc /bin/sh -c "cc -o tmp tmp.s; ./tmp"

    actual="$?"

    echo -e "${YELLOW}\`\`\`$input\`\`\`${NC}"
    if [ "$actual" = "$expected" ]; then
        echo "=> $actual"
        echo
    else
        echo -e "${RED}=> $expected expected, but got $actual${NC}"
        echo
        exit 1
    fi
}

# assert 42 '42;'
# assert 47 '5+6*7;'
# assert 15 '5*(9-6);'
# assert 4 '(3+5)/2;'
# assert 10 '-10+20;'
# assert 1 '-5--6;' 
# assert 19 '-5-+6+30;'
# assert 1 '(4 == 2) + (3 < 5);'
# assert 1 '6 >= 2;'
# assert 1 '6 != 2;'
# assert 4 '
# a = 4;
# a;
# '
# assert 6 '
# foo = 1;
# bar = 2 + 3;
# foo + bar;
# '
# assert 7 '
# ZoanfA_5ad = (4 * 4) / 2;
# fh978a__4A = 2 - 3;
# ZoanfA_5ad + fh978a__4A;
# '
# assert 14 '
# a = 3;
# b = 5 * 6 - 8;
# return a + b / 2;
# '
# assert 5 '
# return 5;
# return 8;
# '
# assert 3 '
# if (1) 3; else 2;
# '
# assert 4 '
# if (1) return 4; 7;
# '
# assert 1 '
# year = 2024;

# year_quarter = year / 4;
# if (year == year_quarter * 4)
#     1;
# else
#     0;
# '
# assert 22 '
# sum = 0;
# while (sum <= 20)
#     sum = sum + 2;
# sum;
# '
assert 55 '
sum = 0;
i = 0;
for (; i <= 10; i = i + 1)
    sum = sum + i;
return sum;
'

echo -e "${GREEN}test finished successfully.${NC}"