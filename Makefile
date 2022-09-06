# CFLAGS : Cコンパイルのオプションを指定
# -std=c11 : 言語標準をc11としてコンパイル
CFLAGS=-std=c11 -g -static

dcc: 
	cargo build

test: dcc
	./test.sh

clean:
	cargo clean
	rm -f *.o tmp*

# ダミーターゲットの指定
.PHONY: dcc test clean
