use crate::{
    error,
    parser::{Node, NodeKind, NodeList},
};

// x86-64に従った関数呼び出しの引数レジスタ
const ARGS: [&str; 6] = ["rdi", "rsi", "rdx", "rcx", "r8", "r9"];

// ユニークな数を出力するためのカウンター
pub struct Counter {
    cnt: usize,
}
impl Counter {
    pub fn new() -> Self {
        Counter { cnt: 0 }
    }
    fn new_cnt(&mut self) -> String {
        let ret = self.cnt.to_string();
        self.cnt += 1;
        ret
    }
}

// 与えられたノードが変数を指しているときに、その変数のアドレスを計算して、その結果をスタックにpushする
fn gen_lval(node: &Node, input: &Vec<char>) {
    if node.kind != NodeKind::Lvar {
        error::error(node.input_idx, "代入の左辺値が変数ではありません", input);
    }

    println!("  mov rax, rbp");
    println!("  sub rax, {}", node.offset.unwrap());
    println!("  push rax");
}

// ASTからスタックマシンをemulateする形でアセンブリを出力する
pub fn gen(now: usize, node_list: &NodeList, input: &Vec<char>, counter: &mut Counter) {
    let now_node = &node_list.nodes[now];

    match now_node.kind {
        NodeKind::If => {
            /*
            if (A) B else C

            - if (A) B の場合

            if (A == 0)
                goto end;
            B;
            end:

            - if (A) B else C の場合

            if (A == 0)
                goto els;
            B;
            goto end;
            els:
            C;
            end:
            */
            let label_name = counter.new_cnt();
            let lhs = &node_list.nodes[now_node.lhs.unwrap()];
            let rhs = &node_list.nodes[now_node.rhs.unwrap()];
            let else_exist = if let Some(_) = rhs.rhs { true } else { false };

            // Aのコード出力
            if lhs.kind != NodeKind::IfFlag {
                error::error(lhs.input_idx, "IFの判定部分が期待されています", input);
            }
            gen(lhs.lhs.unwrap(), node_list, input, counter);

            // Aの結果をpopして分岐
            println!("  pop rax");
            println!("  cmp rax, 0");
            println!(
                "  je .L{}{}",
                if else_exist { "else" } else { "end" },
                label_name
            );

            // Bのコード出力
            if rhs.kind != NodeKind::IfStmt {
                error::error(rhs.input_idx, "IFのstatementが期待されています", input);
            }
            gen(rhs.lhs.unwrap(), node_list, input, counter);

            // Cのコード出力
            if else_exist {
                println!("  jmp .Lend{}", label_name);
                println!(".Lelse{}:", label_name);
                gen(rhs.rhs.unwrap(), node_list, input, counter);
            }

            println!(".Lend{}:", label_name);
            return;
        }
        NodeKind::While => {
            /*
            while (A) B

            begin:
            if (A == 0)
                goto end;
            B;
            goto begin;
            end:
            */
            let label_name = counter.new_cnt();

            println!(".Lbegin{}:", label_name);
            gen(now_node.lhs.unwrap(), node_list, input, counter); // Aのコード
            println!("  pop rax");
            println!("  cmp rax, 0");
            println!("  je .Lend{}", label_name);
            gen(now_node.rhs.unwrap(), node_list, input, counter); // Bのコード
            println!("  jmp .Lbegin{}", label_name);
            println!(".Lend{}:", label_name);
            return;
        }
        NodeKind::For => {
            /*
            for (A; B; C) D

            A;
            begin:
            if (B == 0)
                goto end;
            D;
            C;
            goto begin;
            end:
            */
            let label_name = counter.new_cnt();
            let lhs = &node_list.nodes[now_node.lhs.unwrap()];
            let rhs = &node_list.nodes[now_node.rhs.unwrap()];

            // Aのコード出力
            if let Some(a) = lhs.lhs {
                gen(a, node_list, input, counter);
            }

            println!(".Lbegin{}:", label_name);

            // Bのコード出力
            if let Some(b) = lhs.rhs {
                gen(b, node_list, input, counter);
            }

            println!("  pop rax");
            println!("  cmp rax, 0");
            println!("  je .Lend{}", label_name);

            // Dのコード出力
            gen(rhs.rhs.unwrap(), node_list, input, counter);

            // Cのコード出力
            if let Some(c) = rhs.lhs {
                gen(c, node_list, input, counter);
            }

            println!("  jmp .Lbegin{}", label_name);
            println!(".Lend{}:", label_name);
            return;
        }
        NodeKind::Return => {
            gen(now_node.lhs.unwrap(), node_list, input, counter);
            println!("  pop rax");
            println!("  mov rsp, rbp");
            println!("  pop rbp");
            println!("  ret");
            return;
        }
        NodeKind::Num => {
            println!("  push {}", now_node.val.unwrap());
            return;
        }
        NodeKind::Lvar => {
            gen_lval(now_node, input);
            println!("  pop rax"); // 左辺値のアドレスを取り出す
            println!("  mov rax, [rax]"); // 左辺値を取り出す
            println!("  push rax");
            return;
        }
        NodeKind::Assign => {
            gen_lval(&node_list.nodes[now_node.lhs.unwrap()], input);
            gen(now_node.rhs.unwrap(), node_list, input, counter);
            println!("  pop rdi"); // 右辺値を取り出す
            println!("  pop rax"); // 左辺値のアドレスを取り出す
            println!("  mov [rax], rdi");
            println!("  push rdi"); // 代入した値をpushしておく
            return;
        }
        NodeKind::Block => {
            let mut node = now_node; // 一番最初のblockノード

            // blockノードのlhsがNoneになるまでループ
            while let Some(stmt) = node.lhs {
                println!("  pop rax"); // 各1つのstmtは1つの値をスタックに残したままにしているので、stmtが続くときは前の値を取り出しておく
                gen(stmt, node_list, input, counter);
                node = &node_list.nodes[node.rhs.unwrap()]; // 次のblockノードをセット
            }

            return;
        }
        NodeKind::App => {
            let lhs = &node_list.nodes[now_node.lhs.unwrap()];
            let mut node = now_node.rhs;

            // nodeがNoneになるまでループ
            let mut arg_idx = 0;

            while let Some(x) = node {
                // 引数レジスタの制限を超えた場合
                if arg_idx >= ARGS.len() {
                    error::error(
                        node_list.nodes[x].input_idx,
                        "これ以上引数を増やせません",
                        input,
                    );
                }

                // nodeがNoneでなかったので、lhsに引数がある
                let expr = node_list.nodes[x].lhs.unwrap();
                gen(expr, node_list, input, counter); // exprを計算するコードを出力
                println!("  pop rax"); // exprの値をraxに取り出す
                println!("  mov {}, rax", ARGS[arg_idx]); // ABIに従ったレジスタに引数を登録
                arg_idx += 1;
                node = node_list.nodes[x].rhs;
            }

            // 関数呼び出しの際はrspが16の倍数になっていなければならないことに注意しながら、関数を呼び出す
            let label_name = counter.new_cnt();
            println!("  mov rax, rsp");
            println!("  and rax, 15");
            println!("  cmp rax, 0");
            println!("  je .LskipAlign{}", label_name);
            println!("  sub rsp, 8");
            println!("  call {}", lhs.var_name.as_ref().unwrap());
            println!("  add rsp, 8");
            println!("  jmp .LendAlign{}", label_name);
            println!(".LskipAlign{}:", label_name);
            println!("  call {}", lhs.var_name.as_ref().unwrap());
            println!(".LendAlign{}:", label_name);

            println!("  push rax"); // 関数の返り値をpush
            return;
        }
        _ => (),
    }

    gen(now_node.lhs.unwrap(), node_list, input, counter);
    gen(now_node.rhs.unwrap(), node_list, input, counter);

    println!("  pop rdi");
    println!("  pop rax");

    match now_node.kind {
        NodeKind::Add => {
            println!("  add rax, rdi");
        }
        NodeKind::Sub => {
            println!("  sub rax, rdi");
        }
        NodeKind::Mul => {
            println!("  imul rax, rdi");
        }
        NodeKind::Div => {
            println!("  cqo");
            println!("  idiv rdi");
        }
        NodeKind::Eq => {
            println!("  cmp rax, rdi");
            println!("  sete al");
            println!("  movzb rax, al");
        }
        NodeKind::Lt => {
            println!("  cmp rax, rdi");
            println!("  setl al");
            println!("  movzb rax, al");
        }
        NodeKind::Le => {
            println!("  cmp rax, rdi");
            println!("  setle al");
            println!("  movzb rax, al");
        }
        NodeKind::Ne => {
            println!("  cmp rax, rdi");
            println!("  setne al");
            println!("  movzb rax, al");
        }
        _ => {
            panic!("unreachable");
        }
    }

    println!("  push rax");
}
