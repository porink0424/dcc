dcc: 
	cargo build

test: dcc
	./test.sh

clean:
	cargo clean
	rm -f *.o tmp*

# ダミーターゲットの指定
.PHONY: dcc test clean
