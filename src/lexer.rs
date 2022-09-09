use crate::error;

// トークンの種類
#[derive(PartialEq, Debug)]
pub enum TokenKind {
    Reserved,
    Num,
    EOF,
}
// トークン型
#[derive(Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub val: Option<isize>,
    pub input_idx: usize, // このTokenがはじまる部分のinputされた文字列のindex
    pub len: usize,       // トークンの長さ
}
#[derive(Debug)]
pub struct TokenList {
    pub now: usize, // 今着目しているトークンのindex
    pub input: Vec<char>,
    pub tokens: Vec<Token>,
}

impl TokenList {
    fn new(p: &Vec<char>) -> Self {
        TokenList {
            now: 0,
            input: p.clone(),
            tokens: vec![],
        }
    }

    pub fn tokenize(p: &Vec<char>) -> Self {
        let mut token_list = Self::new(p);

        let mut idx = 0;
        while idx < p.len() {
            // 空白文字はスキップ
            if p[idx] == ' ' {
                idx += 1;
                continue;
            }

            // 2文字の記号
            if idx + 1 < p.len()
                && ["<=", ">=", "==", "!="]
                    .iter()
                    .any(|c| *c.to_string() == p[idx..=idx + 1].iter().collect::<String>())
            {
                token_list.append_new_token(TokenKind::Reserved, idx, None, 2);
                idx += 2;
                continue;
            }

            // 1文字の記号
            if "+-*/()<>".chars().any(|c| c == p[idx]) {
                token_list.append_new_token(TokenKind::Reserved, idx, None, 1);
                idx += 1;
                continue;
            }

            if (p[idx]).is_numeric() {
                // 数字が終わるところまでループ
                let mut digit_idx = idx + 1;
                while digit_idx < p.len() && p[digit_idx].is_numeric() {
                    digit_idx += 1;
                }
                token_list.append_new_token(
                    TokenKind::Num,
                    idx,
                    Some(
                        p[idx..digit_idx]
                            .iter()
                            .collect::<String>()
                            .parse()
                            .unwrap(),
                    ),
                    digit_idx - idx,
                );
                idx = digit_idx;
                continue;
            }

            error::error(idx, "tokenizeできません", p);
        }

        token_list.append_new_token(TokenKind::EOF, idx, None, 0);
        token_list
    }

    fn get_now_token(&self) -> &Token {
        &(self.tokens[self.now])
    }

    // 次のトークンが期待している記号だったときには、トークンを1つ読み進めてtrueを返す。それ以外はfalseを返す。
    pub fn consume(&mut self, op: &str) -> bool {
        let now_token = self.get_now_token();
        if now_token.kind != TokenKind::Reserved
            || !(self.input[now_token.input_idx..(now_token.input_idx + now_token.len)]
                .iter()
                .collect::<String>()
                .eq(&op.to_string()))
        {
            return false;
        } else {
            self.now += 1;
            return true;
        }
    }

    // 次のトークンが期待している記号だったときには、トークンを1つ読み進める。それ以外はエラーになる。
    pub fn expect(&mut self, op: &str) {
        let now_token = self.get_now_token();
        if now_token.kind != TokenKind::Reserved
            || !(self.input[now_token.input_idx..(now_token.input_idx + now_token.len)]
                .iter()
                .collect::<String>()
                .eq(&op.to_string()))
        {
            error::error(
                now_token.input_idx,
                format!("'{}'ではありません", op).as_str(),
                &self.input,
            );
        } else {
            self.now += 1;
        }
    }

    // 次のトークンが数値の場合、トークンを1つ読み進めてその数値を返す。それ以外はエラーになる。
    pub fn expect_number(&mut self) -> Option<isize> {
        let now_token = self.get_now_token();
        if now_token.kind != TokenKind::Num {
            error::error(now_token.input_idx, "数ではありません", &self.input);
        }
        let val = now_token.val;
        self.now += 1;
        val
    }

    fn append_new_token(
        &mut self,
        kind: TokenKind,
        input_idx: usize,
        val: Option<isize>,
        len: usize,
    ) {
        self.tokens.push(Token {
            kind,
            val: if let Some(_) = val { val } else { None },
            input_idx,
            len,
        })
    }
}
