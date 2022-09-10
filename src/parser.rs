use crate::{error, lexer::TokenList};

// ローカル変数の型
#[derive(Debug)]
struct LVar {
    name: String,  // 名前の長さ
    offset: usize, // RBPからのオフセット
}
#[derive(Debug)]
pub struct LVarList {
    lvars: Vec<LVar>,
}
impl LVarList {
    fn new() -> Self {
        LVarList { lvars: vec![] }
    }

    // 変数を名前で検索する。見つからなかった場合はfalseを返す
    fn find_lvar(&self, name: &String) -> (Option<&LVar>, bool) {
        for lvar in self.lvars.iter() {
            if name.eq(&lvar.name) {
                return (Some(lvar), true);
            }
        }
        (None, false)
    }
}

// ノードの種類
#[derive(PartialEq, Debug)]
pub enum NodeKind {
    ADD,    // +
    SUB,    // -
    MUL,    // *
    DIV,    // /
    EQ,     // ==
    NE,     // !=
    LT,     // <
    LE,     // <=
    ASSIGN, // =
    LVAR,   // local int
    NUM,    // int
    RETURN, // return
}
// ノード型
#[derive(Debug)]
pub struct Node {
    pub kind: NodeKind,
    pub input_idx: usize,      // 入力のうち、このノードが始まる場所のindex
    pub lhs: Option<usize>,    // 左辺のノードのindex
    pub rhs: Option<usize>,    // 左辺のノードのindex
    pub val: Option<isize>,    // kindがNUMの時のみ利用
    pub offset: Option<usize>, // kindがLVARの時のみ利用。ローカル変数のベースポインタからのオフセットを表す。
}
#[derive(Debug)]
pub struct NodeList {
    pub roots: Vec<usize>, // プログラムの中の各文のrootノードのindex
    pub nodes: Vec<Node>,
    pub lval_list: LVarList,
}
impl NodeList {
    pub fn new() -> Self {
        NodeList {
            roots: vec![],
            nodes: vec![],
            lval_list: LVarList::new(),
        }
    }

    // 新しいノードを作成し、そのindexを返す
    fn append_new_node(
        &mut self,
        kind: NodeKind,
        input_idx: usize,
        lhs: usize,
        rhs: usize,
    ) -> usize {
        let new_idx = self.nodes.len();
        self.nodes.push(Node {
            kind,
            input_idx,
            lhs: Some(lhs),
            rhs: Some(rhs),
            val: None,
            offset: None,
        });
        new_idx
    }

    // 新しい数字ノードを作成し、そのindexを返す
    fn append_new_node_num(
        &mut self,
        input_idx: usize,
        val: Option<isize>,
        token_list: &TokenList,
    ) -> usize {
        if let None = val {
            error::error(input_idx, "数ではありません", &token_list.input);
        }
        let new_idx = self.nodes.len();
        self.nodes.push(Node {
            kind: NodeKind::NUM,
            input_idx,
            lhs: None,
            rhs: None,
            val,
            offset: None,
        });
        new_idx
    }

    // 新しいローカル変数ノードを作成し、そのindexを返す
    fn append_new_node_lvar(
        &mut self,
        input_idx: usize,
        offset: Option<usize>,
        token_list: &TokenList,
    ) -> usize {
        if let None = offset {
            error::error(input_idx, "ローカル変数ではありません", &token_list.input);
        }
        let new_idx = self.nodes.len();
        self.nodes.push(Node {
            kind: NodeKind::LVAR,
            input_idx,
            lhs: None,
            rhs: None,
            val: None,
            offset,
        });
        new_idx
    }

    // 新しいreturnノードを作成し、そのindexを返す
    fn append_new_node_return(&mut self, input_idx: usize, lhs: Option<usize>) -> usize {
        let new_idx = self.nodes.len();
        self.nodes.push(Node {
            kind: NodeKind::RETURN,
            input_idx,
            lhs,
            rhs: None,
            val: None,
            offset: None,
        });
        new_idx
    }

    // program    = stmt*
    pub fn program(&mut self, token_list: &mut TokenList) {
        while !token_list.at_eof() {
            let idx = self.stmt(token_list);
            self.roots.push(idx);
        }
    }

    // stmt    = expr ";" | "return" expr ";"
    fn stmt(&mut self, token_list: &mut TokenList) -> usize {
        let idx;
        let input_idx = token_list.tokens[token_list.now].input_idx;
        if token_list.consume_return() {
            let lhs = self.expr(token_list);
            idx = self.append_new_node_return(input_idx, Some(lhs));
        } else {
            idx = self.expr(token_list);
        }
        token_list.expect(";");
        idx
    }

    // expr       = assign
    fn expr(&mut self, token_list: &mut TokenList) -> usize {
        self.assign(token_list)
    }

    // assign     = equality ("=" assign)?
    fn assign(&mut self, token_list: &mut TokenList) -> usize {
        let mut idx = self.equality(token_list);
        if token_list.consume("=") {
            let rhs = self.assign(token_list);
            let input_idx = token_list.tokens[token_list.now].input_idx;
            idx = self.append_new_node(NodeKind::ASSIGN, input_idx, idx, rhs);
        }
        idx
    }

    // equality   = relational ("==" relational | "!=" relational)*
    fn equality(&mut self, token_list: &mut TokenList) -> usize {
        let mut idx = self.relational(token_list);

        let input_idx = token_list.tokens[token_list.now].input_idx;
        loop {
            if token_list.consume("==") {
                let rhs = self.relational(token_list);
                idx = self.append_new_node(NodeKind::EQ, input_idx, idx, rhs);
            } else if token_list.consume("!=") {
                let rhs = self.relational(token_list);
                idx = self.append_new_node(NodeKind::NE, input_idx, idx, rhs);
            } else {
                return idx;
            }
        }
    }

    // relational = add ("<" add | "<=" add | ">" add | ">=" add)*
    fn relational(&mut self, token_list: &mut TokenList) -> usize {
        let mut idx = self.add(token_list);

        let input_idx = token_list.tokens[token_list.now].input_idx;
        loop {
            if token_list.consume("<") {
                let rhs = self.add(token_list);
                idx = self.append_new_node(NodeKind::LT, input_idx, idx, rhs);
            } else if token_list.consume("<=") {
                let rhs = self.add(token_list);
                idx = self.append_new_node(NodeKind::LE, input_idx, idx, rhs);
            } else if token_list.consume(">") {
                let lhs = self.add(token_list);
                idx = self.append_new_node(NodeKind::LT, input_idx, lhs, idx);
            } else if token_list.consume(">=") {
                let lhs = self.add(token_list);
                idx = self.append_new_node(NodeKind::LE, input_idx, lhs, idx);
            } else {
                return idx;
            }
        }
    }

    // add        = mul ("+" mul | "-" mul)*
    fn add(&mut self, token_list: &mut TokenList) -> usize {
        let mut idx = self.mul(token_list);

        let input_idx = token_list.tokens[token_list.now].input_idx;
        loop {
            if token_list.consume("+") {
                let rhs = self.mul(token_list);
                idx = self.append_new_node(NodeKind::ADD, input_idx, idx, rhs);
            } else if token_list.consume("-") {
                let rhs = self.mul(token_list);
                idx = self.append_new_node(NodeKind::SUB, input_idx, idx, rhs);
            } else {
                return idx;
            }
        }
    }

    // mul     = unary ("*" unary | "/" unary)*
    fn mul(&mut self, token_list: &mut TokenList) -> usize {
        let mut idx = self.unary(token_list);

        let input_idx = token_list.tokens[token_list.now].input_idx;
        loop {
            if token_list.consume("*") {
                let rhs = self.unary(token_list);
                idx = self.append_new_node(NodeKind::MUL, input_idx, idx, rhs);
            } else if token_list.consume("/") {
                let rhs = self.unary(token_list);
                idx = self.append_new_node(NodeKind::DIV, input_idx, idx, rhs);
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
            let input_idx = token_list.tokens[token_list.now].input_idx;
            let zero = self.append_new_node_num(input_idx, Some(0), token_list);
            let rhs = self.primary(token_list);
            self.append_new_node(NodeKind::SUB, input_idx, zero, rhs)
        } else {
            self.primary(token_list)
        }
    }

    // primary    = num | ident | "(" expr ")"
    fn primary(&mut self, token_list: &mut TokenList) -> usize {
        let input_idx = token_list.tokens[token_list.now].input_idx;
        if token_list.consume("(") {
            // 次のトークンが'('なら'(expr)'
            let idx = self.expr(token_list);
            token_list.expect(")");
            idx
        } else if let (Some(token_ident), true) = token_list.consume_ident() {
            // ident
            let token_ident_idx = token_ident.input_idx;
            let token_ident_len = token_ident.len;
            let var_name: String = token_list.input
                [token_ident_idx..(token_ident_idx + token_ident_len)]
                .iter()
                .collect();
            if let (Some(lvar), true) = self.lval_list.find_lvar(&var_name) {
                // 今までに使われたことがある
                self.append_new_node_lvar(input_idx, Some(lvar.offset), token_list)
            } else {
                // idx番目に新しいローカル変数を、lvar_listに登録する
                let idx = self.lval_list.lvars.len();
                self.lval_list.lvars.push(LVar {
                    name: var_name,
                    offset: (idx + 1) * 8,
                });
                self.append_new_node_lvar(input_idx, Some((idx + 1) * 8), token_list)
            }
        } else {
            // num
            self.append_new_node_num(input_idx, token_list.expect_number(), token_list)
        }
    }
}
