#!/bin/bash
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
PURPLE='\033[0;35m'
NC='\033[0m'

echo
echo -e "${PURPLE}test started.${NC}"
echo

# sample.oを生成
docker run --rm -v $(cd $(dirname $0) && pwd):/dcc -w /dcc dcc /bin/sh -c "cc -c sample.c"

assert() {
    expected="$1"
    input="$2"

    # dccによるコンパイル
    ./target/debug/dcc r "$input" > tmp.s
    # アセンブル、sample.oとのリンク、実行をdocker環境でやらせる
    docker run --rm -v $(cd $(dirname $0) && pwd):/dcc -w /dcc dcc /bin/sh -c "cc -c tmp.s; cc -o tmp tmp.o sample.o; ./tmp"

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

# assert 15 '5*(9-6);'
# assert 4 '(3+5)/2;'
# assert 19 '-5-+6+30;'
# assert 1 '(4 == 2) + (3 < 5);'
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
# assert 55 '
# sum = 0;
# i = 0;
# for (; i <= 10; i = i + 1)
#     sum = sum + i;
# return sum;
# '
# # numberが素数なら1を出力するプログラム
# assert 1 '
# number = 13;
# flag = 0;

# for (i = 2; i < number; i = i + 1) {
#     if ((number / i) * i == number) {
#         flag = 1;
#     }
# }

# if (flag) {
#     return 0;
# } else {
#     return 1;
# }
# '
# assert 21 '
# main() {
# a = add(1,2,3,4,5,6);
# return a;
# }
# '
# assert 66 '
# add(x, y) {
#     print(x);
#     print(y);
#     return x + y;
# }

# main() {
#     sum = 0;
#     for (i = 0; i <= 10; i = i + 2) {
#         sum = sum + add(i, i + 1);
#     }
#     return sum;
# }
# '
assert 55 '
fib(n) {
    if (n == 1) {
        return 1;
    } else if (n == 2) {
        return 1;
    } else {
        return fib(n-1) + fib(n-2);
    }
}

main() {
    return fib(10);
}
'

echo -e "${GREEN}test finished successfully.${NC}"