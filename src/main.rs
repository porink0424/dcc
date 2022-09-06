use std::env;

fn main() {
    let args = env::args().collect::<Vec<String>>();

    if args.len() != 2 {
        panic!("引数の個数が正しくありません。");
    }

    println!(".intel_syntax noprefix");
    println!(".global main");
    println!("main:");
    println!("  mov rax, {}", args[1].parse::<isize>().unwrap());
    println!("  ret");
}
