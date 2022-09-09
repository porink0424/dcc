mod codegen;
mod error;
mod lexer;
mod parser;
use std::env;

fn main() {
    // 実行時引数からプログラムを受け取る
    let args = env::args().collect::<Vec<String>>();
    if args.len() != 2 {
        eprintln!("引数の個数が正しくありません");
        std::process::exit(1);
    }

    // 字句解析
    let mut token_list = lexer::TokenList::tokenize(&args[1].chars().collect());

    // 構文解析
    let mut node_list = parser::NodeList::new();
    let root_node_idx = node_list.parse(&mut token_list);

    // アセンブリの前半部分を出力
    println!(".intel_syntax noprefix");
    println!(".global main");
    println!("main:");

    // ASTをトップダウンに降りコード出力
    codegen::gen(root_node_idx, &node_list);

    // スタックトップに残っている式の最終的な値をraxにロードして終了
    println!("  pop rax");
    println!("  ret");
}
