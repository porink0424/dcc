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
    str: Vec<char>,
}
#[derive(Debug)]
struct TokenList {
    now: usize, // 今着目しているトークンのindex
    tokens: Vec<Token>,
}

impl TokenList {
    fn new() -> Self {
        TokenList {
            now: 0,
            tokens: vec![],
        }
    }

    fn tokenize(p: &Vec<char>) -> Self {
        let mut token_list = Self::new();

        let mut idx = 0;
        while idx < p.len() {
            // 空白文字はスキップ
            if p[idx] == ' ' {
                idx += 1;
                continue;
            }

            if p[idx] == '+' || p[idx] == '-' {
                token_list.append_new_token(TokenKind::TKReserved, vec![p[idx]], None);
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
                    p[idx..digit_idx].to_vec(),
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

            error("tokenizeできません");
        }

        token_list.append_new_token(TokenKind::TKEOF, vec![], None);
        token_list
    }

    fn get_now_token(&self) -> &Token {
        &(self.tokens[self.now])
    }

    // 次のトークンが期待している記号だったときには、トークンを1つ読み進めてtrueを返す。それ以外はfalseを返す。
    fn consume(&mut self, op: char) -> bool {
        let now_token = self.get_now_token();
        if now_token.kind != TokenKind::TKReserved || now_token.str[0] != op {
            return false;
        } else {
            self.now += 1;
            return true;
        }
    }

    // 次のトークンが期待している記号だったときには、トークンを1つ読み進める。それ以外はエラーになる。
    fn expect(&mut self, op: char) {
        let now_token = self.get_now_token();
        if now_token.kind != TokenKind::TKReserved || now_token.str[0] != op {
            error(format!("'{}'ではありません", op).as_str());
        } else {
            self.now += 1;
        }
    }

    // 次のトークンが数値の場合、トークンを1つ読み進めてその数値を返す。それ以外はエラーになる。
    fn expect_number(&mut self) -> Option<isize> {
        let now_token = self.get_now_token();
        if now_token.kind != TokenKind::TKNum {
            error("数ではありません");
        }
        let val = now_token.val;
        self.now += 1;
        val
    }

    fn at_eof(&self) -> bool {
        let now_token = self.get_now_token();
        now_token.kind == TokenKind::TKEOF
    }

    fn append_new_token(&mut self, kind: TokenKind, str: Vec<char>, val: Option<isize>) {
        self.tokens.push(Token {
            kind,
            val: if let Some(_) = val { val } else { None },
            str,
        })
    }
}

// エラー報告用の関数
fn error(fmt: &str) {
    println!("{}", fmt);
    std::process::exit(1);
}

fn main() {
    let args = env::args().collect::<Vec<String>>();

    if args.len() != 2 {
        error("引数の個数が正しくありません");
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
