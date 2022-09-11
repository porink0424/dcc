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
    // println!("{:#?}", token_list);

    // 構文解析
    let mut node_list = parser::NodeList::new();
    node_list.program(&mut token_list);
    // println!("{:#?}", node_list);

    // アセンブリの前半部分を出力
    println!(".intel_syntax noprefix");
    println!(".global main");
    println!("main:");

    // 変数26個分の領域をメモリ上に確保
    println!("  push rbp");
    println!("  mov rbp, rsp");
    println!("  sub rsp, 208");

    // ASTをトップダウンに降りコード出力
    let mut counter = codegen::Counter::new();
    for root in node_list.roots.iter() {
        codegen::gen(*root, &node_list, &token_list.input, &mut counter);
        println!("  pop rax"); // スタックがいっぱいにならないように毎回raxにpopする
    }

    // スタックトップに残っている式の最終的な値をraxにロードして終了
    println!("  mov rsp, rbp");
    println!("  pop rbp");
    println!("  ret");
}
