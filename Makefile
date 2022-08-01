# CFLAGS : Cコンパイルのオプションを指定
# -std=c11 : 言語標準をc11としてコンパイル
CFLAGS=-std=c11 -g -static

# 単純にmakeと打った場合は、一番最初のターゲットのみがビルドされる（おそらく）

# Makefileは
# ビルドするファイル（ターゲット）: 依存するファイル
# 		ビルドのためのコマンド
# という形式でかかれる
dcc: dcc.c

# 依存するファイルとしてdccが書かれているので、dccがない場合は勝手にmake dccを走らせてくれる
test: dcc
	./test.sh

# *~ means all files ending in ~. Many Unix/Linux systems programs create backup files that end in ~.
clean:
	rm -f dcc *.o *~ tmp*

# ダミーターゲットの指定
# test, cleanというファイルを作りたいわけではないので、test, cleanをダミーターゲットとして指定
.PHONY: test clean
