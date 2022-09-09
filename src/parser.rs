use crate::{
    error,
    lexer::{TokenKind, TokenList},
};

// ノードの種類
#[derive(PartialEq, Debug)]
pub enum NodeKind {
    ADD, // +
    SUB, // -
    MUL, // *
    DIV, // /
    EQ,  // ==
    NE,  // !=
    LT,  // <
    LE,  // <=
    NUM,
}
// ノード型
#[derive(Debug)]
pub struct Node {
    pub kind: NodeKind,
    pub lhs: Option<usize>, // 左辺のノードのindex
    pub rhs: Option<usize>, // 左辺のノードのindex
    pub val: Option<isize>, // kindがNUMの時のみ利用
}
#[derive(Debug)]
pub struct NodeList {
    pub nodes: Vec<Node>,
}

impl NodeList {
    pub fn new() -> Self {
        NodeList { nodes: vec![] }
    }

    // 新しいノードを作成し、そのindexを返す
    fn append_new_node(&mut self, kind: NodeKind, lhs: usize, rhs: usize) -> usize {
        let new_idx = self.nodes.len();
        self.nodes.push(Node {
            kind,
            lhs: Some(lhs),
            rhs: Some(rhs),
            val: None,
        });
        new_idx
    }

    // 新しい数字ノードを作成し、そのindexを返す
    fn append_new_node_num(&mut self, val: Option<isize>, token_list: &TokenList) -> usize {
        if let None = val {
            error::error(
                token_list.tokens[token_list.now].input_idx,
                "数ではありません",
                &token_list.input,
            );
        }
        let new_idx = self.nodes.len();
        self.nodes.push(Node {
            kind: NodeKind::NUM,
            lhs: None,
            rhs: None,
            val,
        });
        new_idx
    }

    pub fn parse(&mut self, token_list: &mut TokenList) -> usize {
        let ret = self.expr(token_list);
        self.parse_check(token_list);
        ret
    }

    // expr       = equality
    fn expr(&mut self, token_list: &mut TokenList) -> usize {
        self.equality(token_list)
    }

    // equality   = relational ("==" relational | "!=" relational)*
    fn equality(&mut self, token_list: &mut TokenList) -> usize {
        let mut idx = self.relational(token_list);

        loop {
            if token_list.consume("==") {
                let rhs = self.relational(token_list);
                idx = self.append_new_node(NodeKind::EQ, idx, rhs);
            } else if token_list.consume("!=") {
                let rhs = self.relational(token_list);
                idx = self.append_new_node(NodeKind::NE, idx, rhs);
            } else {
                return idx;
            }
        }
    }

    // relational = add ("<" add | "<=" add | ">" add | ">=" add)*
    fn relational(&mut self, token_list: &mut TokenList) -> usize {
        let mut idx = self.add(token_list);

        loop {
            if token_list.consume("<") {
                let rhs = self.add(token_list);
                idx = self.append_new_node(NodeKind::LT, idx, rhs);
            } else if token_list.consume("<=") {
                let rhs = self.add(token_list);
                idx = self.append_new_node(NodeKind::LE, idx, rhs);
            } else if token_list.consume(">") {
                let lhs = self.add(token_list);
                idx = self.append_new_node(NodeKind::LT, lhs, idx);
            } else if token_list.consume(">=") {
                let lhs = self.add(token_list);
                idx = self.append_new_node(NodeKind::LE, lhs, idx);
            } else {
                return idx;
            }
        }
    }

    // add        = mul ("+" mul | "-" mul)*
    fn add(&mut self, token_list: &mut TokenList) -> usize {
        let mut idx = self.mul(token_list);

        loop {
            if token_list.consume("+") {
                let rhs = self.mul(token_list);
                idx = self.append_new_node(NodeKind::ADD, idx, rhs);
            } else if token_list.consume("-") {
                let rhs = self.mul(token_list);
                idx = self.append_new_node(NodeKind::SUB, idx, rhs);
            } else {
                return idx;
            }
        }
    }

    // mul     = unary ("*" unary | "/" unary)*
    fn mul(&mut self, token_list: &mut TokenList) -> usize {
        let mut idx = self.unary(token_list);

        loop {
            if token_list.consume("*") {
                let rhs = self.unary(token_list);
                idx = self.append_new_node(NodeKind::MUL, idx, rhs);
            } else if token_list.consume("/") {
                let rhs = self.unary(token_list);
                idx = self.append_new_node(NodeKind::DIV, idx, rhs);
            } else {
                return idx;
            }
        }
    }

    // unary   = ("+" | "-")? primary
    fn unary(&mut self, token_list: &mut TokenList) -> usize {
        if token_list.consume("+") {
            self.primary(token_list)
        } else if token_list.consume("-") {
            // -nは0-nに置き換える
            let zero = self.append_new_node_num(Some(0), token_list);
            let rhs = self.primary(token_list);
            self.append_new_node(NodeKind::SUB, zero, rhs)
        } else {
            self.primary(token_list)
        }
    }

    // primary = num | "(" expr ")"
    fn primary(&mut self, token_list: &mut TokenList) -> usize {
        if token_list.consume("(") {
            // 次のトークンが'('なら'(expr)'なはず
            let idx = self.expr(token_list);
            token_list.expect(")");
            idx
        } else {
            // そうでなければ数値なはず
            self.append_new_node_num(token_list.expect_number(), token_list)
        }
    }

    // 最後までトークンを読み切ることができなかった場合はエラー
    fn parse_check(&mut self, token_list: &mut TokenList) {
        if token_list.tokens[token_list.now].kind != TokenKind::EOF {
            error::error(
                token_list.tokens[token_list.now].input_idx,
                "文法に合いません",
                &token_list.input,
            );
        }
    }
}
