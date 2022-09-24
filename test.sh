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
    if [ $? = 0 ]; then
        # コンパイル成功した場合

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
    else
    # コンパイル失敗した場合
        echo -e "${YELLOW}\`\`\`$input\`\`\`${NC}"
        echo -e "${RED}=> compile error"
        echo
        exit 1
    fi
}

# assert 13 '
# int main() {
#     int ZoanfA_5ad;
#     int fh978a__4A; 
#     ZoanfA_5ad = (4 * 4) / 2;
#     fh978a__4A = 2 - 3;
#     return (ZoanfA_5ad + fh978a__4A) * 2 - 1;
# }
# '
# assert 1 '
# int main() {
#     int year;
#     int year_quarter;

#     year = 2024;

#     year_quarter = year / 4;
#     if (year == year_quarter * 4)
#         1;
#     else
#         0;
# }
# '
# assert 22 '
# int main() {
#     int sum;
#     sum = 0;
#     while (sum <= 20)
#         sum = sum + 2;
#     sum;
# }
# '
# assert 55 '
# int main() {
#     int sum;
#     int i;
#     sum = 0;
#     i = 0;
#     for (; i <= 10; i = i + 1)
#         sum = sum + i;
#     return sum;
# }
# '
# # numberが素数なら1を出力するプログラム
# assert 1 '
# int main() {
#     int number;
#     number = 13;

#     int flag;
#     flag = 0;

#     int i;

#     for (i = 2; i < number; i = i + 1) {
#         if ((number / i) * i == number) {
#             flag = 1;
#         }
#     }

#     if (flag) {
#         return 0;
#     } else {
#         return 1;
#     }
# }
# '
# assert 66 '
# int add(int x, int y) {
#     return x + y;
# }

# int main() {
#     int i;
#     int sum;

#     sum = 0;
#     for (i = 0; i <= 10; i = i + 2) {
#         sum = sum + add(i, i + 1);
#     }
#     return sum;
# }
# '
# assert 55 '
# int fib(int n) {
#     if (n == 1) {
#         return 1;
#     } else if (n == 2) {
#         return 1;
#     } else {
#         return fib(n-1) + fib(n-2);
#     }
# }

# int main() {
#     return fib(10);
# }
# '
# assert 3 '
# int main() {
#     int x;
#     int *y;
#     x = 4;
#     y = &x;
#     *y = 3;
#     return x;
# }
# '
# assert 4 '
# int assign(int *x) {
#     *x = 4;
# }

# int main() {
#     int x;
#     int *y;
#     x = 3;
#     y = &x;
#     assign(y);
#     return x;
# }
# '
# assert 12 '
# int main () {
#     int x;
#     int y;
#     x = 82;
#     y = 12;
#     int *z;
#     z = &x;
#     z = z - 2;
#     return *z;
# }
# '
# assert 4 '
# int main() {
#     int *p;
#     alloc4(&p, 1, 2, 4, 8);
#     int *q;
#     q = p + 2;
#     return *q;
# }
# '
assert 0 '
int main() {
    int x;
    int *y;

    print(sizeof(x));
    print(sizeof(y));

    print(sizeof(x+3));
    print(sizeof(y+3));
    print(sizeof(*y));

    print(sizeof(1));

    print(sizeof(sizeof(1)));

    return 0;
}
'

echo -e "${GREEN}test finished successfully.${NC}"