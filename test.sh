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
riscv64-unknown-elf-gcc -c sample.c

assert() {
    expected="$1"
    input="$2"

    # dccによるコンパイル
    ./target/debug/dcc r "$input" > tmp.s
    if [ $? = 0 ]; then
        # コンパイル成功した場合

        # アセンブル
        riscv64-unknown-elf-gcc -c tmp.s

        # sample.oとのリンク
        riscv64-unknown-elf-gcc -o tmp tmp.o sample.o

        # 実行
        spike pk tmp

        # 実行結果の代入
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

assert 0 'aa'

echo -e "${GREEN}test finished successfully.${NC}"