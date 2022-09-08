use std::env;

// トークンの種類
#[derive(PartialEq, Debug)]
enum TokenKind {
    TKReserved,
    TKNum,
    TKEOF,
}

// トークン型
#[derive(Debug)]
struct Token {
    kind: TokenKind,
    val: Option<isize>,
    input_idx: usize, // このTokenがはじまる部分のinputされた文字列のindex
}
#[derive(Debug)]
struct TokenList {
    now: usize, // 今着目しているトークンのindex
    input: Vec<char>,
    tokens: Vec<Token>,
}

impl TokenList {
    fn new(p: &Vec<char>) -> Self {
        TokenList {
            now: 0,
            input: p.clone(),
            tokens: vec![],
        }
    }

    fn tokenize(p: &Vec<char>) -> Self {
        let mut token_list = Self::new(p);

        let mut idx = 0;
        while idx < p.len() {
            // 空白文字はスキップ
            if p[idx] == ' ' {
                idx += 1;
                continue;
            }

            if p[idx] == '+' || p[idx] == '-' {
                token_list.append_new_token(TokenKind::TKReserved, idx, None);
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
                    TokenKind::TKNum,
                    idx,
                    Some(
                        p[idx..digit_idx]
                            .iter()
                            .collect::<String>()
                            .parse()
                            .unwrap(),
                    ),
                );
                idx = digit_idx;
                continue;
            }

            error(idx, "tokenizeできません", p);
        }

        token_list.append_new_token(TokenKind::TKEOF, idx, None);
        token_list
    }

    fn get_now_token(&self) -> &Token {
        &(self.tokens[self.now])
    }

    // 次のトークンが期待している記号だったときには、トークンを1つ読み進めてtrueを返す。それ以外はfalseを返す。
    fn consume(&mut self, op: char) -> bool {
        let now_token = self.get_now_token();
        if now_token.kind != TokenKind::TKReserved || self.input[now_token.input_idx] != op {
            return false;
        } else {
            self.now += 1;
            return true;
        }
    }

    // 次のトークンが期待している記号だったときには、トークンを1つ読み進める。それ以外はエラーになる。
    fn expect(&mut self, op: char) {
        let now_token = self.get_now_token();
        if now_token.kind != TokenKind::TKReserved || self.input[now_token.input_idx] != op {
            error(
                now_token.input_idx,
                format!("'{}'ではありません", op).as_str(),
                &self.input,
            );
        } else {
            self.now += 1;
        }
    }

    // 次のトークンが数値の場合、トークンを1つ読み進めてその数値を返す。それ以外はエラーになる。
    fn expect_number(&mut self) -> Option<isize> {
        let now_token = self.get_now_token();
        if now_token.kind != TokenKind::TKNum {
            error(now_token.input_idx, "数ではありません", &self.input);
        }
        let val = now_token.val;
        self.now += 1;
        val
    }

    fn at_eof(&self) -> bool {
        let now_token = self.get_now_token();
        now_token.kind == TokenKind::TKEOF
    }

    fn append_new_token(&mut self, kind: TokenKind, input_idx: usize, val: Option<isize>) {
        self.tokens.push(Token {
            kind,
            val: if let Some(_) = val { val } else { None },
            input_idx,
        })
    }
}

// エラー報告用の関数
fn error(loc: usize, fmt: &str, p: &Vec<char>) {
    eprintln!("{}", p.iter().collect::<String>());
    eprintln!("{}^", " ".to_string().repeat(loc));
    eprintln!("{}", fmt);

    std::process::exit(1);
}

fn main() {
    let args = env::args().collect::<Vec<String>>();

    if args.len() != 2 {
        eprintln!("引数の個数が正しくありません");
    }

    let mut token_list = TokenList::tokenize(&args[1].chars().collect());

    // アセンブリの前半部分を出力
    println!(".intel_syntax noprefix");
    println!(".global main");
    println!("main:");

    // 式の最初は数字
    println!("  mov rax, {}", token_list.expect_number().unwrap());

    // + 数字、もしくは - 数字という並びをひたすら消費していく
    while !token_list.at_eof() {
        if token_list.consume('+') {
            println!("  add rax, {}", token_list.expect_number().unwrap());
            continue;
        }

        // '+'ではなかったので'-'が必ずくるはず
        token_list.expect('-');
        println!("  sub rax, {}", token_list.expect_number().unwrap());
    }

    println!("  ret");
}
