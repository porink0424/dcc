use crate::{
    error,
    lexer::{TokenKind, TokenList},
    typ::{binary_calc_type, match_assign_type},
};

// 変数の型
#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Type {
    Int(usize), // ポインタの段数をusizeで持つ。例えばInt(2)はint **型を表す
    Unknown,
    Stmt, // 文には型がない。構文の維持のために使われるノードが持つ
}

// ローカル変数の型
#[derive(Debug)]
pub struct LVar {
    pub name: String,  // 名前
    pub offset: usize, // RBPからのオフセット
    pub typ: Type,     // 型
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
    pub fn find_lvar(&self, name: &String) -> (Option<&LVar>, bool) {
        for lvar in self.lvars.iter() {
            if name.eq(&lvar.name) {
                return (Some(lvar), true);
            }
        }
        (None, false)
    }

    // 新しい変数を追加する。オフセットを返り値として返す
    fn add_new_lvar(&mut self, name: &String, typ: Type) {
        let len = self.lvars.len();
        self.lvars.push(LVar {
            name: name.clone(),
            offset: (len + 1) * 8,
            typ,
        });
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
    App,   // 関数適用 <- rhsには連続的にArgノードを持つ
    Arg,   // lhsにexprからなるノードを、rhsに連続的にArgノードを持つ
    Addr,  // 単項&
    Deref, // 単項*
    Int,   // ローカル変数定義
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
    pub name: Option<String>,  // kindがLVAR, APPの時のみ利用。ローカル変数, 関数の名前を表す。
    pub typ: Type,
}
#[derive(Debug)]
pub struct NodeList {
    pub roots: Vec<usize>, // プログラムの中の各文のrootノードのindex
    pub nodes: Vec<Node>,
    pub lvar_list: LVarList,
}
impl NodeList {
    pub fn new(args: &Vec<(String, Type)>) -> Self {
        // 関数定義の引数として与えられた変数は、そのような変数が最初から存在するものとしてコンパイルしておく
        let mut lvar_list = LVarList::new();
        for (arg_name, arg_type) in args.iter() {
            lvar_list.add_new_lvar(arg_name, *arg_type);
        }

        NodeList {
            roots: vec![],
            nodes: vec![],
            lvar_list,
        }
    }

    // 新しいノードを作成し、そのindexを返す
    fn append_new_node(
        &mut self,
        kind: NodeKind,
        input_idx: usize,
        lhs: Option<usize>,
        rhs: Option<usize>,
        name: Option<String>,
        typ: Type,
    ) -> usize {
        let new_idx = self.nodes.len();
        self.nodes.push(Node {
            kind,
            input_idx,
            lhs: if let Some(_) = lhs { lhs } else { None },
            rhs: if let Some(_) = rhs { rhs } else { None },
            name: if let Some(_) = name { name } else { None },
            val: None,
            offset: None,
            typ,
        });
        new_idx
    }

    // 新しい数字ノードを作成し、そのindexを返す
    fn append_new_node_num(
        &mut self,
        input_idx: usize,
        val: Option<isize>,
        token_list: &TokenList,
        typ: Type,
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
            name: None,
            typ,
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
        typ: Type,
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
            name: Some(var_name.clone()),
            typ,
        });
        new_idx
    }

    /*
    stmt    = expr ";"
            | "{" stmt* "}"
            | "int" "*"* ident ";"
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
                None,
                Type::Stmt,
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
                    None,
                    Type::Stmt,
                );
                self.nodes[prev_block_node_idx].lhs = Some(lhs);
                self.nodes[prev_block_node_idx].rhs = Some(block_node_idx);
            }
        } else if token_list.consume(TokenKind::Return, None) {
            // return
            let (lhs, _) = self.expr(token_list);
            idx = self.append_new_node(
                NodeKind::Return,
                input_idx,
                Some(lhs),
                None,
                None,
                Type::Stmt,
            );
            token_list.expect(TokenKind::Reserved, Some(";"));
        } else if token_list.consume(TokenKind::Int, None) {
            // intの変数定義
            let mut nst = 0;
            while token_list.consume(TokenKind::Reserved, Some("*")) {
                nst += 1;
            }
            let (ident, res) = token_list.consume_ident();
            if !res {
                error::error(
                    input_idx,
                    "変数の定義式を正しくパースできません",
                    &token_list.input,
                );
                unreachable!()
            }
            let ident = ident.unwrap();
            let token_ident_idx = ident.input_idx;
            let token_ident_len = ident.len;
            let var_name: String = token_list.input
                [token_ident_idx..(token_ident_idx + token_ident_len)]
                .iter()
                .collect();
            self.lvar_list.add_new_lvar(&var_name, Type::Int(nst));
            idx = self.append_new_node(
                NodeKind::Int,
                token_list.tokens[token_list.now].input_idx,
                None,
                None,
                Some(var_name),
                Type::Stmt,
            );
            token_list.expect(TokenKind::Reserved, Some(";"));
        } else if token_list.consume(TokenKind::If, None) {
            // if
            token_list.expect(TokenKind::Reserved, Some("("));
            let (flag, _) = self.expr(token_list);
            let input_idx_inner = token_list.tokens[token_list.now].input_idx; // '('
            let lhs = self.append_new_node(
                NodeKind::IfFlag,
                input_idx_inner,
                Some(flag),
                None,
                None,
                Type::Stmt,
            );
            token_list.expect(TokenKind::Reserved, Some(")"));
            let input_idx_inner = token_list.tokens[token_list.now].input_idx; // ')'
            let stmt_left = Some(self.stmt(token_list));
            let mut stmt_right = None;
            if token_list.consume(TokenKind::Else, None) {
                // else
                stmt_right = Some(self.stmt(token_list));
            }
            let rhs = self.append_new_node(
                NodeKind::IfStmt,
                input_idx_inner + 1,
                stmt_left,
                stmt_right,
                None,
                Type::Stmt,
            );
            idx = self.append_new_node(
                NodeKind::If,
                input_idx,
                Some(lhs),
                Some(rhs),
                None,
                Type::Stmt,
            );
        } else if token_list.consume(TokenKind::While, None) {
            // while
            token_list.expect(TokenKind::Reserved, Some("("));
            let (expr, _) = self.expr(token_list);
            token_list.expect(TokenKind::Reserved, Some(")"));
            let stmt = self.stmt(token_list);
            idx = self.append_new_node(
                NodeKind::While,
                input_idx,
                Some(expr),
                Some(stmt),
                None,
                Type::Stmt,
            );
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
                forfst_lhs = Some(self.expr(token_list).0);
                token_list.consume(TokenKind::Reserved, Some(";"));
            }
            // 2つ目のexpr
            if !token_list.consume(TokenKind::Reserved, Some(";")) {
                forfst_rhs = Some(self.expr(token_list).0);
                token_list.consume(TokenKind::Reserved, Some(";"));
            }
            // 2つめの';'
            let forsnd_lhs_input_idx = token_list.now;
            // 3つ目のexpr
            if !token_list.consume(TokenKind::Reserved, Some(")")) {
                forsnd_lhs = Some(self.expr(token_list).0);
                token_list.consume(TokenKind::Reserved, Some(")"));
            }
            forsnd_rhs = Some(self.stmt(token_list));
            let lhs = self.append_new_node(
                NodeKind::ForFst,
                forfst_lhs_input_idx,
                forfst_lhs,
                forfst_rhs,
                None,
                Type::Stmt,
            );
            let rhs = self.append_new_node(
                NodeKind::ForSnd,
                forsnd_lhs_input_idx,
                forsnd_lhs,
                forsnd_rhs,
                None,
                Type::Stmt,
            );
            idx = self.append_new_node(
                NodeKind::For,
                input_idx,
                Some(lhs),
                Some(rhs),
                None,
                Type::Stmt,
            );
        } else {
            let res = self.expr(token_list);
            idx = res.0;
            token_list.expect(TokenKind::Reserved, Some(";"));
        }
        idx
    }

    // expr以降は、nodeのindexだけではなく、型も返す

    // expr       = assign
    fn expr(&mut self, token_list: &mut TokenList) -> (usize, Type) {
        self.assign(token_list)
    }

    // assign     = equality ("=" assign)?
    fn assign(&mut self, token_list: &mut TokenList) -> (usize, Type) {
        let (mut idx, mut typ) = self.equality(token_list);
        if token_list.consume(TokenKind::Reserved, Some("=")) {
            let (rhs, ty) = self.assign(token_list);

            match_assign_type(typ, ty, token_list);

            typ = ty;
            let input_idx = token_list.tokens[token_list.now].input_idx;
            idx = self.append_new_node(
                NodeKind::Assign,
                input_idx,
                Some(idx),
                Some(rhs),
                None,
                ty, // 代入演算子の返り値は代入した値そのもの
            );
        }
        (idx, typ)
    }

    // equality   = relational ("==" relational | "!=" relational)*
    fn equality(&mut self, token_list: &mut TokenList) -> (usize, Type) {
        let (mut idx, mut typ) = self.relational(token_list);

        let input_idx = token_list.tokens[token_list.now].input_idx;
        loop {
            if token_list.consume(TokenKind::Reserved, Some("==")) {
                let (rhs, ty) = self.relational(token_list);
                typ = ty;
                idx = self.append_new_node(
                    NodeKind::Eq,
                    input_idx,
                    Some(idx),
                    Some(rhs),
                    None,
                    Type::Int(0), // 比較演算子の返り値は1or0のINT
                );
            } else if token_list.consume(TokenKind::Reserved, Some("!=")) {
                let (rhs, ty) = self.relational(token_list);
                typ = ty;
                idx = self.append_new_node(
                    NodeKind::Ne,
                    input_idx,
                    Some(idx),
                    Some(rhs),
                    None,
                    Type::Int(0),
                );
            } else {
                break;
            }
        }
        (idx, typ)
    }

    // relational = add ("<" add | "<=" add | ">" add | ">=" add)*
    fn relational(&mut self, token_list: &mut TokenList) -> (usize, Type) {
        let (mut idx, mut typ) = self.add(token_list);

        let input_idx = token_list.tokens[token_list.now].input_idx;
        loop {
            if token_list.consume(TokenKind::Reserved, Some("<")) {
                let (rhs, ty) = self.add(token_list);
                typ = ty;
                idx = self.append_new_node(
                    NodeKind::Lt,
                    input_idx,
                    Some(idx),
                    Some(rhs),
                    None,
                    Type::Int(0),
                );
            } else if token_list.consume(TokenKind::Reserved, Some("<=")) {
                let (rhs, ty) = self.add(token_list);
                typ = ty;
                idx = self.append_new_node(
                    NodeKind::Le,
                    input_idx,
                    Some(idx),
                    Some(rhs),
                    None,
                    Type::Int(0),
                );
            } else if token_list.consume(TokenKind::Reserved, Some(">")) {
                let (lhs, ty) = self.add(token_list);
                typ = ty;
                idx = self.append_new_node(
                    NodeKind::Lt,
                    input_idx,
                    Some(lhs),
                    Some(idx),
                    None,
                    Type::Int(0),
                );
            } else if token_list.consume(TokenKind::Reserved, Some(">=")) {
                let (lhs, ty) = self.add(token_list);
                typ = ty;
                idx = self.append_new_node(
                    NodeKind::Le,
                    input_idx,
                    Some(lhs),
                    Some(idx),
                    None,
                    Type::Int(0),
                );
            } else {
                break;
            }
        }
        (idx, typ)
    }

    // add        = mul ("+" mul | "-" mul)*
    fn add(&mut self, token_list: &mut TokenList) -> (usize, Type) {
        let (mut idx, mut typ) = self.mul(token_list);

        let input_idx = token_list.tokens[token_list.now].input_idx;
        loop {
            if token_list.consume(TokenKind::Reserved, Some("+")) {
                let (rhs, ty) = self.mul(token_list);
                typ = binary_calc_type(typ, ty, token_list);
                idx =
                    self.append_new_node(NodeKind::Add, input_idx, Some(idx), Some(rhs), None, typ);
            } else if token_list.consume(TokenKind::Reserved, Some("-")) {
                let (rhs, ty) = self.mul(token_list);
                typ = binary_calc_type(typ, ty, token_list);
                idx =
                    self.append_new_node(NodeKind::Sub, input_idx, Some(idx), Some(rhs), None, typ);
            } else {
                break;
            }
        }
        (idx, typ)
    }

    // mul     = unary ("*" unary | "/" unary)*
    fn mul(&mut self, token_list: &mut TokenList) -> (usize, Type) {
        let (mut idx, mut typ) = self.unary(token_list);

        let input_idx = token_list.tokens[token_list.now].input_idx;
        loop {
            if token_list.consume(TokenKind::Reserved, Some("*")) {
                let (rhs, ty) = self.unary(token_list);
                typ = binary_calc_type(typ, ty, token_list);
                idx =
                    self.append_new_node(NodeKind::Mul, input_idx, Some(idx), Some(rhs), None, typ);
            } else if token_list.consume(TokenKind::Reserved, Some("/")) {
                let (rhs, ty) = self.unary(token_list);
                typ = binary_calc_type(typ, ty, token_list);
                idx =
                    self.append_new_node(NodeKind::Div, input_idx, Some(idx), Some(rhs), None, typ);
            } else {
                break;
            }
        }
        (idx, typ)
    }

    // unary   = "sizeof" unary | ("+" | "-")? primary | "*" unary | "&" unary
    fn unary(&mut self, token_list: &mut TokenList) -> (usize, Type) {
        if token_list.consume(TokenKind::Sizeof, None) {
            // sizeof
            let (_idx, typ) = self.unary(token_list);
            let input_idx = token_list.tokens[token_list.now].input_idx;
            match typ {
                Type::Int(0) => (
                    self.append_new_node_num(input_idx, Some(4), token_list, Type::Int(0)),
                    Type::Int(0),
                ),
                Type::Int(x) if x > 0 => (
                    self.append_new_node_num(input_idx, Some(8), token_list, Type::Int(0)),
                    Type::Int(0),
                ),
                _ => unreachable!(),
            }
        } else if token_list.consume(TokenKind::Reserved, Some("+")) {
            // +
            self.primary(token_list)
        } else if token_list.consume(TokenKind::Reserved, Some("-")) {
            // -
            // -nは0-nに置き換える
            let input_idx = token_list.tokens[token_list.now].input_idx;
            let zero = self.append_new_node_num(input_idx, Some(0), token_list, Type::Int(0));
            let (rhs, typ) = self.primary(token_list);
            (
                self.append_new_node(NodeKind::Sub, input_idx, Some(zero), Some(rhs), None, typ),
                typ,
            )
        } else if token_list.consume(TokenKind::Reserved, Some("*")) {
            // deref
            let input_idx = token_list.tokens[token_list.now].input_idx;
            let (lhs, mut typ) = self.unary(token_list);
            match typ {
                Type::Int(x) if x >= 1 => typ = Type::Int(x - 1),
                _ => error::error(input_idx, "dereferenceできません", &token_list.input),
            }
            (
                self.append_new_node(NodeKind::Deref, input_idx, Some(lhs), None, None, typ),
                typ,
            )
        } else if token_list.consume(TokenKind::Reserved, Some("&")) {
            // addr
            let input_idx = token_list.tokens[token_list.now].input_idx;
            let (lhs, mut typ) = self.unary(token_list);
            match typ {
                Type::Int(x) => typ = Type::Int(x + 1),
                _ => error::error(input_idx, "アドレスを取得できません", &token_list.input),
            }
            (
                self.append_new_node(NodeKind::Addr, input_idx, Some(lhs), None, None, typ),
                typ,
            )
        } else {
            self.primary(token_list)
        }
    }

    // primary    = num | ident ("(" expr* ")")? | "(" expr ")"
    fn primary(&mut self, token_list: &mut TokenList) -> (usize, Type) {
        let input_idx = token_list.tokens[token_list.now].input_idx;
        if token_list.consume(TokenKind::Reserved, Some("(")) {
            // 次のトークンが'('なら'(expr)'
            let (idx, typ) = self.expr(token_list);
            token_list.expect(TokenKind::Reserved, Some(")"));
            (idx, typ)
        } else if let (Some(token_ident), true) = token_list.consume_ident() {
            // ident
            let token_ident_idx = token_ident.input_idx;
            let token_ident_len = token_ident.len;
            let var_name: String = token_list.input
                [token_ident_idx..(token_ident_idx + token_ident_len)]
                .iter()
                .collect();
            let ret;
            let typ;
            if let (Some(lvar), true) = self.lvar_list.find_lvar(&var_name) {
                // 今までに使われたことがあるローカル変数
                typ = lvar.typ;
                ret = self.append_new_node_lvar(
                    input_idx,
                    Some(lvar.offset),
                    token_list,
                    &var_name,
                    typ,
                );
            } else if token_list.consume(TokenKind::Reserved, Some("(")) {
                // 関数呼び出し
                ret = self.append_new_node(
                    NodeKind::App,
                    input_idx,
                    None,
                    None,
                    Some(var_name),
                    Type::Unknown,
                );
                let mut node = ret;
                typ = Type::Unknown;
                if token_list.consume(TokenKind::Reserved, Some(")")) {
                    // 引数がない場合はなにもしない
                } else {
                    // 引数が1個以上ある
                    loop {
                        // 引数が続く
                        let (expr, typ) = self.expr(token_list);
                        let arg = self.append_new_node(
                            NodeKind::Arg,
                            token_list.tokens[token_list.now].input_idx,
                            Some(expr),
                            None,
                            None,
                            typ,
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
            } else {
                error::error(
                    input_idx,
                    "定義されていない変数が使われています",
                    &token_list.input,
                );
                unreachable!();
            }

            (ret, typ)
        } else {
            // num
            (
                self.append_new_node_num(
                    input_idx,
                    token_list.expect_number(),
                    token_list,
                    Type::Int(0),
                ),
                Type::Int(0),
            )
        }
    }
}

#[derive(Debug)]
pub struct Func {
    pub program: NodeList,         // 関数をNodeListを用いて表現する
    pub args: Vec<(String, Type)>, // (関数の引数名, 型)
    pub name: String,              // 関数の名前
}
impl Func {
    // func    = "int" "*"* ident "(" ("int" "*"* ident)* ")" "{" stmt* "}"
    pub fn new(token_list: &mut TokenList) -> Self {
        token_list.expect(TokenKind::Int, None);

        // TODO: 関数の返り値の型の利用
        let mut _nst = 0;
        while token_list.consume(TokenKind::Reserved, Some("*")) {
            _nst += 1;
        }

        let input_idx = token_list.tokens[token_list.now].input_idx;

        let func_name = token_list.expect_ident();
        token_list.expect(TokenKind::Reserved, Some("("));

        let mut args = vec![];
        if token_list.consume(TokenKind::Reserved, Some(")")) {
            // 引数が何もない場合はなにもしない
        } else {
            // 引数が1個以上ある
            loop {
                token_list.expect(TokenKind::Int, None);
                let mut nst = 0;
                while token_list.consume(TokenKind::Reserved, Some("*")) {
                    nst += 1;
                }
                let arg_name = token_list.expect_ident();
                args.push((arg_name, Type::Int(nst)));
                if token_list.consume(TokenKind::Reserved, Some(")")) {
                    // 引数は終わり
                    break;
                } else if token_list.consume(TokenKind::Reserved, Some(",")) {
                    // 引数はまだ続く
                    continue;
                } else {
                    // ここには辿り着かないはずなのでparseが失敗している
                    error::error(input_idx, "不正な関数定義です", &token_list.input);
                }
            }
        }

        token_list.expect(TokenKind::Reserved, Some("{"));

        let mut program = NodeList::new(&args);
        while !token_list.consume(TokenKind::Reserved, Some("}")) {
            let idx = program.stmt(token_list);
            program.roots.push(idx);
        }

        Func {
            program,
            args,
            name: func_name,
        }
    }
}
