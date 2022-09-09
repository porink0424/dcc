// エラー報告用の関数
pub fn error(loc: usize, fmt: &str, p: &Vec<char>) {
    eprintln!("{}", p.iter().collect::<String>());
    eprintln!("{}^", " ".to_string().repeat(loc));
    eprintln!("{}", fmt);

    std::process::exit(1);
}
