use crate::{
    error,
    lexer::{TokenKind, TokenList},
};

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
    Add,    // +
    Sub,    // -
    Mul,    // *
    Div,    // /
    Eq,     // ==
    Ne,     // !=
    Lt,     // <
    Le,     // <=
    Assign, // =
    Lvar,   // local int
    Num,    // int
    Return, // return
    If,     // if <- IfFlagとIfStmtをそれぞれlhs, rhsに持つ
    IfFlag,
    IfStmt,
    While, // while <- flagとstmtをそれぞれlhs, rhsに持つ
    For,   // for <- ForFstとForSndをそれぞれlhs, rhsに持つ
    ForFst,
    ForSnd,
    Block, // { ... } <- lhsにはstmtからなるノードを、rhsには連続的にBlockノードを持つ
    App,   // 関数適用 <- lhsに関数名が入ったLvarを持つ。rhsには連続的にArgノードを持つ
    Arg,   // lhsにexprからなるノードを、rhsに連続的にArgノードを持つ
}
// ノード型
#[derive(Debug)]
pub struct Node {
    pub kind: NodeKind,
    pub input_idx: usize,         // 入力のうち、このノードが始まる場所のindex
    pub lhs: Option<usize>,       // 左辺のノードのindex
    pub rhs: Option<usize>,       // 左辺のノードのindex
    pub val: Option<isize>,       // kindがNUMの時のみ利用
    pub offset: Option<usize>, // kindがLVARの時のみ利用。ローカル変数のベースポインタからのオフセットを表す。
    pub var_name: Option<String>, // kindがLVARの時のみ利用。ローカル変数の名前を表す。
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
        lhs: Option<usize>,
        rhs: Option<usize>,
    ) -> usize {
        let new_idx = self.nodes.len();
        self.nodes.push(Node {
            kind,
            input_idx,
            lhs: if let Some(_) = lhs { lhs } else { None },
            rhs: if let Some(_) = rhs { rhs } else { None },
            val: None,
            offset: None,
            var_name: None,
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
            kind: NodeKind::Num,
            input_idx,
            lhs: None,
            rhs: None,
            val,
            offset: None,
            var_name: None,
        });
        new_idx
    }

    // 新しいローカル変数ノードを作成し、そのindexを返す
    fn append_new_node_lvar(
        &mut self,
        input_idx: usize,
        offset: Option<usize>,
        token_list: &TokenList,
        var_name: &String,
    ) -> usize {
        if let None = offset {
            error::error(input_idx, "ローカル変数ではありません", &token_list.input);
        }
        let new_idx = self.nodes.len();
        self.nodes.push(Node {
            kind: NodeKind::Lvar,
            input_idx,
            lhs: None,
            rhs: None,
            val: None,
            offset,
            var_name: Some(var_name.clone()),
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

    /*
    stmt    = expr ";"
            | "{" stmt* "}"
            | "if" "(" expr ")" stmt ("else" stmt)?
            | "while" "(" expr ")" stmt
            | "for" "(" expr? ";" expr? ";" expr? ")" stmt
            | "return" expr ";"
    */
    fn stmt(&mut self, token_list: &mut TokenList) -> usize {
        let idx;
        let input_idx = token_list.tokens[token_list.now].input_idx;
        if token_list.consume(TokenKind::Reserved, Some("{")) {
            // compound statement
            let mut block_node_idx = self.append_new_node(
                NodeKind::Block,
                token_list.tokens[token_list.now].input_idx,
                None,
                None,
            );
            idx = block_node_idx; // 一番上のblockノードを最終的に返す
            while !token_list.consume(TokenKind::Reserved, Some("}")) {
                let lhs = self.stmt(token_list);
                let prev_block_node_idx = block_node_idx;
                block_node_idx = self.append_new_node(
                    NodeKind::Block,
                    token_list.tokens[token_list.now].input_idx,
                    None,
                    None,
                );
                self.nodes[prev_block_node_idx].lhs = Some(lhs);
                self.nodes[prev_block_node_idx].rhs = Some(block_node_idx);
            }
        } else if token_list.consume(TokenKind::Return, None) {
            // return
            let lhs = self.expr(token_list);
            idx = self.append_new_node(NodeKind::Return, input_idx, Some(lhs), None);
            token_list.expect(TokenKind::Reserved, Some(";"));
        } else if token_list.consume(TokenKind::If, None) {
            // if
            token_list.expect(TokenKind::Reserved, Some("("));
            let flag = self.expr(token_list);
            let input_idx_inner = token_list.tokens[token_list.now].input_idx; // '('
            let lhs = self.append_new_node(NodeKind::IfFlag, input_idx_inner, Some(flag), None);
            token_list.expect(TokenKind::Reserved, Some(")"));
            let input_idx_inner = token_list.tokens[token_list.now].input_idx; // ')'
            let stmt_left = Some(self.stmt(token_list));
            let mut stmt_right = None;
            if token_list.consume(TokenKind::Else, None) {
                // else
                stmt_right = Some(self.stmt(token_list));
            }
            let rhs =
                self.append_new_node(NodeKind::IfStmt, input_idx_inner + 1, stmt_left, stmt_right);
            idx = self.append_new_node(NodeKind::If, input_idx, Some(lhs), Some(rhs));
        } else if token_list.consume(TokenKind::While, None) {
            // while
            token_list.expect(TokenKind::Reserved, Some("("));
            let expr = self.expr(token_list);
            token_list.expect(TokenKind::Reserved, Some(")"));
            let stmt = self.stmt(token_list);
            idx = self.append_new_node(NodeKind::While, input_idx, Some(expr), Some(stmt));
        } else if token_list.consume(TokenKind::For, None) {
            // for
            token_list.expect(TokenKind::Reserved, Some("("));
            // '('
            let forfst_lhs_input_idx = token_list.now;
            let mut forfst_lhs = None;
            let mut forfst_rhs = None;
            let mut forsnd_lhs = None;
            let forsnd_rhs;
            // 1つ目のexpr
            if !token_list.consume(TokenKind::Reserved, Some(";")) {
                forfst_lhs = Some(self.expr(token_list));
                token_list.consume(TokenKind::Reserved, Some(";"));
            }
            // 2つ目のexpr
            if !token_list.consume(TokenKind::Reserved, Some(";")) {
                forfst_rhs = Some(self.expr(token_list));
                token_list.consume(TokenKind::Reserved, Some(";"));
            }
            // 2つめの';'
            let forsnd_lhs_input_idx = token_list.now;
            // 3つ目のexpr
            if !token_list.consume(TokenKind::Reserved, Some(")")) {
                forsnd_lhs = Some(self.expr(token_list));
                token_list.consume(TokenKind::Reserved, Some(")"));
            }
            forsnd_rhs = Some(self.stmt(token_list));
            let lhs = self.append_new_node(
                NodeKind::ForFst,
                forfst_lhs_input_idx,
                forfst_lhs,
                forfst_rhs,
            );
            let rhs = self.append_new_node(
                NodeKind::ForSnd,
                forsnd_lhs_input_idx,
                forsnd_lhs,
                forsnd_rhs,
            );
            idx = self.append_new_node(NodeKind::For, input_idx, Some(lhs), Some(rhs));
        } else {
            idx = self.expr(token_list);
            token_list.expect(TokenKind::Reserved, Some(";"));
        }
        idx
    }

    // expr       = assign
    fn expr(&mut self, token_list: &mut TokenList) -> usize {
        self.assign(token_list)
    }

    // assign     = equality ("=" assign)?
    fn assign(&mut self, token_list: &mut TokenList) -> usize {
        let mut idx = self.equality(token_list);
        if token_list.consume(TokenKind::Reserved, Some("=")) {
            let rhs = self.assign(token_list);
            let input_idx = token_list.tokens[token_list.now].input_idx;
            idx = self.append_new_node(NodeKind::Assign, input_idx, Some(idx), Some(rhs));
        }
        idx
    }

    // equality   = relational ("==" relational | "!=" relational)*
    fn equality(&mut self, token_list: &mut TokenList) -> usize {
        let mut idx = self.relational(token_list);

        let input_idx = token_list.tokens[token_list.now].input_idx;
        loop {
            if token_list.consume(TokenKind::Reserved, Some("==")) {
                let rhs = self.relational(token_list);
                idx = self.append_new_node(NodeKind::Eq, input_idx, Some(idx), Some(rhs));
            } else if token_list.consume(TokenKind::Reserved, Some("!=")) {
                let rhs = self.relational(token_list);
                idx = self.append_new_node(NodeKind::Ne, input_idx, Some(idx), Some(rhs));
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
            if token_list.consume(TokenKind::Reserved, Some("<")) {
                let rhs = self.add(token_list);
                idx = self.append_new_node(NodeKind::Lt, input_idx, Some(idx), Some(rhs));
            } else if token_list.consume(TokenKind::Reserved, Some("<=")) {
                let rhs = self.add(token_list);
                idx = self.append_new_node(NodeKind::Le, input_idx, Some(idx), Some(rhs));
            } else if token_list.consume(TokenKind::Reserved, Some(">")) {
                let lhs = self.add(token_list);
                idx = self.append_new_node(NodeKind::Lt, input_idx, Some(lhs), Some(idx));
            } else if token_list.consume(TokenKind::Reserved, Some(">=")) {
                let lhs = self.add(token_list);
                idx = self.append_new_node(NodeKind::Le, input_idx, Some(lhs), Some(idx));
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
            if token_list.consume(TokenKind::Reserved, Some("+")) {
                let rhs = self.mul(token_list);
                idx = self.append_new_node(NodeKind::Add, input_idx, Some(idx), Some(rhs));
            } else if token_list.consume(TokenKind::Reserved, Some("-")) {
                let rhs = self.mul(token_list);
                idx = self.append_new_node(NodeKind::Sub, input_idx, Some(idx), Some(rhs));
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
            if token_list.consume(TokenKind::Reserved, Some("*")) {
                let rhs = self.unary(token_list);
                idx = self.append_new_node(NodeKind::Mul, input_idx, Some(idx), Some(rhs));
            } else if token_list.consume(TokenKind::Reserved, Some("/")) {
                let rhs = self.unary(token_list);
                idx = self.append_new_node(NodeKind::Div, input_idx, Some(idx), Some(rhs));
            } else {
                return idx;
            }
        }
    }

    // unary   = ("+" | "-")? primary
    fn unary(&mut self, token_list: &mut TokenList) -> usize {
        if token_list.consume(TokenKind::Reserved, Some("+")) {
            self.primary(token_list)
        } else if token_list.consume(TokenKind::Reserved, Some("-")) {
            // -nは0-nに置き換える
            let input_idx = token_list.tokens[token_list.now].input_idx;
            let zero = self.append_new_node_num(input_idx, Some(0), token_list);
            let rhs = self.primary(token_list);
            self.append_new_node(NodeKind::Sub, input_idx, Some(zero), Some(rhs))
        } else {
            self.primary(token_list)
        }
    }

    // primary    = num | ident ("(" expr* ")")? | "(" expr ")"
    fn primary(&mut self, token_list: &mut TokenList) -> usize {
        let input_idx = token_list.tokens[token_list.now].input_idx;
        if token_list.consume(TokenKind::Reserved, Some("(")) {
            // 次のトークンが'('なら'(expr)'
            let idx = self.expr(token_list);
            token_list.expect(TokenKind::Reserved, Some(")"));
            idx
        } else if let (Some(token_ident), true) = token_list.consume_ident() {
            // ident
            let token_ident_idx = token_ident.input_idx;
            let token_ident_len = token_ident.len;
            let var_name: String = token_list.input
                [token_ident_idx..(token_ident_idx + token_ident_len)]
                .iter()
                .collect();
            let mut ret;
            if let (Some(lvar), true) = self.lval_list.find_lvar(&var_name) {
                // 今までに使われたことがある
                ret = self.append_new_node_lvar(input_idx, Some(lvar.offset), token_list, &var_name)
            } else {
                // idx番目に新しいローカル変数を、lvar_listに登録する
                let idx = self.lval_list.lvars.len();
                ret = self.append_new_node_lvar(
                    input_idx,
                    Some((idx + 1) * 8),
                    token_list,
                    &var_name,
                );
                self.lval_list.lvars.push(LVar {
                    name: var_name,
                    offset: (idx + 1) * 8,
                });
            }

            if token_list.consume(TokenKind::Reserved, Some("(")) {
                // 関数呼び出し
                ret = self.append_new_node(NodeKind::App, input_idx, Some(ret), None);
                let mut node = ret;

                if token_list.consume(TokenKind::Reserved, Some(")")) {
                    // 引数がない場合はなにもしない
                } else {
                    // 引数が1個以上ある
                    loop {
                        // 引数が続く
                        let expr = self.expr(token_list);
                        let arg = self.append_new_node(
                            NodeKind::Arg,
                            token_list.tokens[token_list.now].input_idx,
                            Some(expr),
                            None,
                        );
                        self.nodes[node].rhs = Some(arg);
                        node = arg;
                        if token_list.consume(TokenKind::Reserved, Some(")")) {
                            // 引数は終わり
                            break;
                        } else if token_list.consume(TokenKind::Reserved, Some(",")) {
                            // 引数はまだ続く
                            continue;
                        } else {
                            // ここには辿り着かないはずなのでparseが失敗している
                            error::error(input_idx, "不正な関数呼び出しです", &token_list.input);
                        }
                    }
                }
            }

            ret
        } else {
            // num
            self.append_new_node_num(input_idx, token_list.expect_number(), token_list)
        }
    }
}
