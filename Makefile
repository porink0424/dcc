dcc: 
	cargo build

test: dcc
	./test.sh

clean:
	cargo clean
	rm -f *.o tmp*

# アセンブリを手で書いて確かめてみたいときに利用
ex:
	riscv64-unknown-elf-gcc -c sample.c
	riscv64-unknown-elf-gcc -c ex.s
	riscv64-unknown-elf-gcc -o tmp ex.o sample.o
	spike pk tmp

# ダミーターゲットの指定
.PHONY: dcc test clean dcc
