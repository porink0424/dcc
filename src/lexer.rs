use std::vec;

use crate::{
    common::{Input, Line},
    error::Error,
};

// トークンの種類
#[derive(PartialEq, Debug)]
pub enum TokenKind {
    Reserved { sign: String }, // 記号
    ID { name: String },       // 識別子
    Num { val: isize },        // 整数トークン
    If,                        // if
    Else,                      // else
    While,                     // while
    For,                       // for
    Return,                    // リターン
    Sizeof,                    // sizeof
    Int,
    Char,
    EOF,
}
// トークン型
#[derive(Debug)]
pub struct Token {
    pub row: usize,
    pub col_start: usize,
    pub col_end: usize,
    pub kind: TokenKind,
}

// プログラムを表すトークン列
#[derive(Debug)]
pub struct TokenList {
    pub now: usize, // 今着目しているトークンのindex
    pub input: Input,
    pub tokens: Vec<Token>,
}
impl TokenList {
    fn new(input: Input) -> Self {
        TokenList {
            now: 0,
            input,
            tokens: vec![],
        }
    }

    // lineにおいて、line[idx]からwordというTokenを作ることができるか判定し、作れるときtoken_listに足す。
    // Tokenを作った場合、Some(new_idx)、そうでないときNoneを返す
    fn can_tokenize_word(
        &mut self,
        line: &Line,
        row: usize,
        idx: usize,
        word: &String,
        kind: TokenKind,
    ) -> Option<usize> {
        let len = word.len();
        if idx + (len - 1) < line.len()
            && line[idx..(idx + len)].iter().collect::<String>().eq(word)
            && (idx + len >= line.len()
                || !matches!(line[idx + len], 'a'..='z' | 'A'..='Z' | '_' | '0'..='9'))
        {
            self.tokens.push(Token {
                row,
                col_start: idx,
                col_end: idx + len,
                kind,
            });
            Some(idx + len)
        } else {
            None
        }
    }

    pub fn tokenize(p: &Vec<char>) -> (Self, Error) {
        // 文字列からInput型を生成する
        let mut input: Input = vec![];
        let mut line: Line = vec![];
        for c in p.iter() {
            if *c == '\n' {
                input.push(line);
                line = vec![];
            } else {
                line.push(*c);
            }
        }
        input.push(line);

        // Error型の作成
        let error = Error::new(&input);

        // 新しいTokenList型の作成
        let mut token_list = Self::new(input);

        for (row, line) in token_list.input.iter().enumerate() {
            let mut idx = 0;

            while idx < line.len() {
                // 空白文字はスキップ
                if line[idx] == ' ' {
                    idx += 1;
                    continue;
                }

                // if
                if let Some(new_idx) =
                    token_list.can_tokenize_word(line, row, idx, &"if".to_string(), TokenKind::If)
                {
                    idx = new_idx;
                    continue;
                }

                // else
                if let Some(new_idx) = token_list.can_tokenize_word(
                    line,
                    row,
                    idx,
                    &"else".to_string(),
                    TokenKind::Else,
                ) {
                    idx = new_idx;
                    continue;
                }

                // for
                if let Some(new_idx) =
                    token_list.can_tokenize_word(line, row, idx, &"for".to_string(), TokenKind::For)
                {
                    idx = new_idx;
                    continue;
                }

                // while
                if let Some(new_idx) = token_list.can_tokenize_word(
                    line,
                    row,
                    idx,
                    &"while".to_string(),
                    TokenKind::While,
                ) {
                    idx = new_idx;
                    continue;
                }

                // int
                if let Some(new_idx) =
                    token_list.can_tokenize_word(line, row, idx, &"int".to_string(), TokenKind::Int)
                {
                    idx = new_idx;
                    continue;
                }

                // return
                if let Some(new_idx) = token_list.can_tokenize_word(
                    line,
                    row,
                    idx,
                    &"return".to_string(),
                    TokenKind::Return,
                ) {
                    idx = new_idx;
                    continue;
                }

                // sizeof
                if let Some(new_idx) = token_list.can_tokenize_word(
                    line,
                    row,
                    idx,
                    &"sizeof".to_string(),
                    TokenKind::Sizeof,
                ) {
                    idx = new_idx;
                    continue;
                }

                // 2文字の記号
                if idx + 1 < line.len() {
                    if let Some(&sign) = ["<=", ">=", "==", "!="]
                        .iter()
                        .find(|&&c| c.to_string() == line[idx..=idx + 1].iter().collect::<String>())
                    {
                        token_list.tokens.push(Token {
                            row,
                            col_start: idx,
                            col_end: idx + 2,
                            kind: TokenKind::Reserved {
                                sign: sign.to_string(),
                            },
                        });
                        idx += 2;
                        continue;
                    }
                }

                // 1文字の記号
                if idx + 1 < line.len() {
                    if let Some(sign) = "+-*/()<>=;{},&[]".chars().find(|&c| c == line[idx]) {
                        token_list.tokens.push(Token {
                            row,
                            col_start: idx,
                            col_end: idx + 1,
                            kind: TokenKind::Reserved {
                                sign: sign.to_string(),
                            },
                        });
                        idx += 1;
                        continue;
                    }
                }

                // 識別子
                if matches!(line[idx], 'a'..='z' | 'A'..='Z') {
                    // アルファベットが終わるところまでループ
                    let mut alpha_idx = idx + 1;
                    while alpha_idx < line.len()
                        && matches!(line[alpha_idx], 'a'..='z' | 'A'..='Z' | '_' | '0'..='9')
                    {
                        alpha_idx += 1;
                    }
                    token_list.tokens.push(Token {
                        row,
                        col_start: idx,
                        col_end: idx + alpha_idx,
                        kind: TokenKind::ID {
                            name: line[idx..=idx + alpha_idx].iter().collect::<String>(),
                        },
                    });
                    idx = alpha_idx;
                    continue;
                }

                // 数字
                if (line[idx]).is_numeric() {
                    // 数字が終わるところまでループ
                    let mut digit_idx = idx + 1;
                    while digit_idx < line.len() && line[digit_idx].is_numeric() {
                        digit_idx += 1;
                    }
                    token_list.tokens.push(Token {
                        row,
                        col_start: idx,
                        col_end: idx + digit_idx,
                        kind: TokenKind::Num {
                            val: line[idx..=idx + digit_idx]
                                .iter()
                                .collect::<String>()
                                .parse()
                                .unwrap(),
                        },
                    });
                    idx = digit_idx;
                    continue;
                }

                error.lexer_error(row, idx, idx + 1, &"トークナイズできません".to_string());
            }

            token_list.tokens.push(Token {
                row,
                col_start: idx,
                col_end: idx,
                kind: TokenKind::EOF,
            });
        }
        (token_list, error)
    }

    fn get_now_token(&self) -> &Token {
        &(self.tokens[self.now])
    }

    // 次のトークンが期待しているものだったときには、トークンを1つ読み進めてtrueを返す。それ以外はfalseを返す。
    pub fn consume(&mut self, kind: TokenKind) -> bool {
        if self.get_now_token().kind == kind {
            self.now += 1;
            true
        } else {
            false
        }
    }

    // 次のトークンがローカル変数の場合、トークンを1つ読み進めてそのローカル変数に対応するトークンとtrueを返す。それ以外はfalseを返す。
    pub fn consume_ident(&mut self) -> (Option<&Token>, bool) {
        let now_token = self.get_now_token();
        match now_token.kind {
            TokenKind::ID { name } => {
                self.now += 1;
                (Some(now_token), true)
            }
            _ => (None, false),
        }
    }

    // 次のトークンが期待しているものだったときには、トークンを1つ読み進める。それ以外はエラーになる。
    pub fn expect(&mut self, kind: TokenKind, error: Error) {
        let now_token = self.get_now_token();
        if now_token.kind == kind {
            self.now += 1;
        } else {
            error.lexer_error(
                now_token.row,
                now_token.col_start,
                now_token.col_end,
                &format!("{:?}が期待されています", now_token.kind),
            )
        }
    }

    // 次のトークンがIDの場合、トークンを1つ読み進めてその名前を返す。それ以外はエラーになる。
    pub fn expect_ident(&mut self, error: Error) -> String {
        let now_token = self.get_now_token();
        if let TokenKind::ID { name } = now_token.kind {
            self.now += 1;
            name
        } else {
            error.lexer_error(
                now_token.row,
                now_token.col_start,
                now_token.col_end,
                &"識別子が期待されています".to_string(),
            );
            unreachable!()
        }
    }

    // 次のトークンが数値の場合、トークンを1つ読み進めてその数値を返す。それ以外はエラーになる。
    pub fn expect_number(&mut self, error: Error) -> isize {
        let now_token = self.get_now_token();
        if let TokenKind::Num { val } = now_token.kind {
            self.now += 1;
            val
        } else {
            error.lexer_error(
                now_token.row,
                now_token.col_start,
                now_token.col_end,
                &"数字が期待されています".to_string(),
            );
            unreachable!()
        }
    }

    pub fn at_eof(&self) -> bool {
        self.get_now_token().kind == TokenKind::EOF
    }
}
