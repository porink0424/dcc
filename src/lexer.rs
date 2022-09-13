use crate::error;

// トークンの種類
#[derive(PartialEq, Debug)]
pub enum TokenKind {
    Reserved, // 記号
    ID,       // 識別子
    Num,      // 整数トークン
    If,       // if
    Else,     // else
    While,    // while
    For,      // for
    Return,   // リターン
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

    // p[idx]からwordというトークンを作るべきか判定し、作るべきなときはtoken_listに足す。
    // トークンを作った場合、Some(idx)、そうでない場合Noneを返す。
    fn can_tokenize(
        &mut self,
        p: &Vec<char>,
        idx: usize,
        word: &String,
        kind: TokenKind,
    ) -> Option<usize> {
        // 例えばword = returnのとき、
        // returnとなっていて、かつその後にalnumが続かないならtrue
        // returnxはreturnトークンではなくIDトークンになる必要があるのでfalse
        let len = word.len();
        if idx + (len - 1) < p.len()
            && p[idx..(idx + len)].iter().collect::<String>().eq(word)
            && (idx + len >= p.len()
                || !matches!(p[idx + len], 'a'..='z' | 'A'..='Z' | '_' | '0'..='9'))
        {
            self.append_new_token(kind, idx, None, len);
            Some(idx + len)
        } else {
            None
        }
    }

    pub fn tokenize(p: &Vec<char>) -> Self {
        let mut token_list = Self::new(p);

        let mut idx = 0;
        while idx < p.len() {
            // 空白、改行文字はスキップ
            if " \n".chars().any(|c| c == p[idx]) {
                idx += 1;
                continue;
            }

            // if
            if let Some(new_idx) = token_list.can_tokenize(p, idx, &"if".to_string(), TokenKind::If)
            {
                idx = new_idx;
                continue;
            }

            // else
            if let Some(new_idx) =
                token_list.can_tokenize(p, idx, &"else".to_string(), TokenKind::Else)
            {
                idx = new_idx;
                continue;
            }

            // for
            if let Some(new_idx) =
                token_list.can_tokenize(p, idx, &"for".to_string(), TokenKind::For)
            {
                idx = new_idx;
                continue;
            }

            // while
            if let Some(new_idx) =
                token_list.can_tokenize(p, idx, &"while".to_string(), TokenKind::While)
            {
                idx = new_idx;
                continue;
            }

            // return
            if let Some(new_idx) =
                token_list.can_tokenize(p, idx, &"return".to_string(), TokenKind::Return)
            {
                idx = new_idx;
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
            if "+-*/()<>=;{},".chars().any(|c| c == p[idx]) {
                token_list.append_new_token(TokenKind::Reserved, idx, None, 1);
                idx += 1;
                continue;
            }

            // 複数の文字からなる識別子
            if matches!(p[idx], 'a'..='z' | 'A'..='Z') {
                // アルファベットが終わるところまでループ
                let mut alpha_idx = idx + 1;
                while alpha_idx < p.len()
                    && matches!(p[alpha_idx], 'a'..='z' | 'A'..='Z' | '_' | '0'..='9')
                {
                    alpha_idx += 1;
                }
                token_list.append_new_token(TokenKind::ID, idx, None, alpha_idx - idx);
                idx = alpha_idx;
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

    // 次のトークンが期待しているものだったときには、トークンを1つ読み進めてtrueを返す。それ以外はfalseを返す。
    // opはkindがTokenKind::Reservedだったときに値を指定する。
    pub fn consume(&mut self, kind: TokenKind, op: Option<&str>) -> bool {
        let now_token = self.get_now_token();
        match kind {
            TokenKind::Reserved => {
                if now_token.kind != TokenKind::Reserved
                    || !(self.input[now_token.input_idx..(now_token.input_idx + now_token.len)]
                        .iter()
                        .collect::<String>()
                        .eq(&op.unwrap().to_string()))
                {
                    false
                } else {
                    self.now += 1;
                    true
                }
            }
            _ => {
                if now_token.kind != kind {
                    false
                } else {
                    self.now += 1;
                    true
                }
            }
        }
    }

    // 次のトークンがローカル変数の場合、トークンを1つ読み進めてそのローカル変数に対応するトークンとtrueを返す。それ以外はfalseを返す。
    pub fn consume_ident(&mut self) -> (Option<&Token>, bool) {
        let now_token = self.get_now_token();
        if now_token.kind == TokenKind::ID {
            self.now += 1;
            (Some(&(self.tokens[self.now - 1])), true)
        } else {
            (None, false)
        }
    }

    // 次のトークンが期待しているものだったときには、トークンを1つ読み進める。それ以外はエラーになる。
    // opはkindがTokenKind::Reservedだったときに値を指定する。
    pub fn expect(&mut self, kind: TokenKind, op: Option<&str>) {
        let now_token = self.get_now_token();
        match kind {
            TokenKind::Reserved => {
                if now_token.kind != TokenKind::Reserved
                    || !(self.input[now_token.input_idx..(now_token.input_idx + now_token.len)]
                        .iter()
                        .collect::<String>()
                        .eq(&op.unwrap().to_string()))
                {
                    error::error(
                        now_token.input_idx,
                        format!("'{}'が期待されています", op.unwrap()).as_str(),
                        &self.input,
                    );
                } else {
                    self.now += 1;
                }
            }
            _ => {
                if now_token.kind != kind {
                    error::error(
                        now_token.input_idx,
                        format!("'{:?}'が期待されています", kind).as_str(),
                        &self.input,
                    )
                } else {
                    self.now += 1;
                }
            }
        }
    }

    // 次のトークンがIDの場合、トークンを1つ読み進めてその名前を返す。それ以外はエラーになる。
    pub fn expect_ident(&mut self) -> String {
        let now_token = self.get_now_token();
        let input_idx = now_token.input_idx;
        if now_token.kind != TokenKind::ID {
            error::error(input_idx, "識別子が来ることが期待されています", &self.input);
        }
        let len = now_token.len;
        self.now += 1;
        self.input[input_idx..(input_idx + len)].iter().collect()
    }

    // 次のトークンが数値の場合、トークンを1つ読み進めてその数値を返す。それ以外はエラーになる。
    pub fn expect_number(&mut self) -> Option<isize> {
        let now_token = self.get_now_token();
        if now_token.kind != TokenKind::Num {
            error::error(
                now_token.input_idx,
                "数がくることが期待されています",
                &self.input,
            );
        }
        let val = now_token.val;
        self.now += 1;
        val
    }

    pub fn at_eof(&self) -> bool {
        let now_token = self.get_now_token();
        now_token.kind == TokenKind::EOF
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
