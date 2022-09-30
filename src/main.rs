mod codegen;
mod common;
mod error;
mod lexer;
mod parser;
mod typ;
use std::env;

use crate::parser::Func;

fn main() {
    // TODO: リリースモードとデバッグモードの取り扱いをうまくやる実装をする

    // 実行時引数からプログラムを受け取る
    let args = env::args().collect::<Vec<String>>();
    let release_mode;
    if args.len() >= 2 && args[1].chars().collect::<String>().eq(&"r".to_string()) {
        // release mode
        release_mode = true;
        if args.len() < 3 {
            eprintln!("引数の個数が正しくありません");
            std::process::exit(1);
        }
    } else {
        // debug mode
        release_mode = false;
    }

    // 字句解析
    let mut token_list;
    let mut error;
    if release_mode {
        (token_list, error) = lexer::TokenList::tokenize(&args[2].chars().collect());
    } else {
        (token_list, error) = lexer::TokenList::tokenize(
            &"
            int main() {
                int a;
                int b;
                a = 2;
                b = 3;
                a + b;
            }
            "
            .chars()
            .collect::<Vec<char>>(),
        );
        // println!("{:#?}", token_list); // printing for debug
    }

    // 構文解析
    let mut func_list: Vec<Func> = vec![];
    while !token_list.at_eof() {
        let new_func = Func::new(&mut token_list);
        func_list.push(new_func);
    }
    if !release_mode {
        println!("{:#?}", func_list); // printing for debug
    }

    // アセンブリのhead部分を出力
    println!(".intel_syntax noprefix");
    println!(".global main");

    // アセンブリ本体を出力
    for func in func_list.iter() {
        codegen::gen(func, &token_list.input);
    }

    return;
}
