## dcc: A mini-C Compiler

### About

dcc is a mini-C (a small subset of C) compiler scratched in Rust. This compiler compiles a mini-C program to an assembly program based on [RISC-V ISA](https://riscv.org/technical/specifications/).

### mini-C Grammer

This compiler parses a program by LL(1).

```
// type definition
type       = ( "int" | "char" ) "*"*

// satement definition
program    = func*
func       = type ident "(" (type ident)* ")" "{" stmt* "}"
stmt       = expr ";"
            | "{" stmt* "}"
            | type ident ("[" num "]")? ";"
            | "if" "(" expr ")" stmt ("else" stmt)?
            | "while" "(" expr ")" stmt
            | "for" "(" expr? ";" expr? ";" expr? ")" stmt
            | "return" expr ";"
expr       = assign
assign     = equality ("=" assign)?
equality   = relational ("==" relational | "!=" relational)*
relational = add ("<" add | "<=" add | ">" add | ">=" add)*
add        = mul ("+" mul | "-" mul)*
mul        = unary ("*" unary | "/" unary)*
unary      = "sizeof" unary | ("+" | "-")? primary | "*" unary | "&" unary
primary    = num | ident ("(" expr* ")")? | "(" expr ")"
```

### Build

```
make
```

### Clean

```
make clean
```

### Compile

```
FILL ME
```

### Run an assembly

You can make an object file by some RISC-V cross-compiler such as [RISC-V GNU Compiler Toolchain](https://github.com/riscv-collab/riscv-gnu-toolchain).

```
riscv64-unknown-elf-gcc your_asm.s -o your_obj
```

And then you can run the file by some RISC-V ISA Simulator such as [Spike](https://github.com/riscv-software-src/riscv-isa-sim).

```
spike pk your_obj
```

### References

- https://www.sigbus.info/compilerbook
