use crate::common::Input;

pub struct Error<'a> {
    input: &'a Input,
}
impl Error<'_> {
    pub fn new(input: &Input) -> Self {
        Self { input }
    }

    pub fn lexer_error(&self, row: usize, col_start: usize, col_end: usize, msg: &String) {
        eprintln!("エラー：{}", msg);
        eprintln!("{} |", " ".repeat((row + 1).to_string().len()));
        eprintln!(
            "{} | {}",
            row + 1,
            self.input[row].iter().collect::<String>()
        );
        eprintln!(
            "{} | {}{}",
            " ".repeat((row + 1).to_string().len()),
            " ".repeat(col_start.to_string().len()),
            "^".repeat((col_end - col_start).to_string().len())
        );
    }
}

pub fn error() {
    eprintln!("エラーが発生しました");

    std::process::exit(1);
}
