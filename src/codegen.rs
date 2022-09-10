use crate::{
    error,
    parser::{Node, NodeKind, NodeList},
};

// 与えられたノードが変数を指しているときに、その変数のアドレスを計算して、その結果をスタックにpushする
fn gen_lval(node: &Node, input: &Vec<char>) {
    if node.kind != NodeKind::LVAR {
        error::error(node.input_idx, "代入の左辺値が変数ではありません", input);
    }

    println!("  mov rax, rbp");
    println!("  sub rax, {}", node.offset.unwrap());
    println!("  push rax");
}

// ASTからスタックマシンをemulateする形でアセンブリを出力する
pub fn gen(now: usize, node_list: &NodeList, input: &Vec<char>) {
    let now_node = &node_list.nodes[now];

    match now_node.kind {
        NodeKind::RETURN => {
            gen(now_node.lhs.unwrap(), node_list, input);
            println!("  pop rax");
            println!("  mov rsp, rbp");
            println!("  pop rbp");
            println!("  ret");
            return;
        }
        NodeKind::NUM => {
            println!("  push {}", now_node.val.unwrap());
            return;
        }
        NodeKind::LVAR => {
            gen_lval(now_node, input);
            println!("  pop rax"); // 左辺値のアドレスを取り出す
            println!("  mov rax, [rax]"); // 左辺値を取り出す
            println!("  push rax");
            return;
        }
        NodeKind::ASSIGN => {
            gen_lval(&node_list.nodes[now_node.lhs.unwrap()], input);
            gen(now_node.rhs.unwrap(), node_list, input);
            println!("  pop rdi"); // 右辺値を取り出す
            println!("  pop rax"); // 左辺値のアドレスを取り出す
            println!("  mov [rax], rdi");
            println!("  push rdi"); // 代入した値をpushしておく
            return;
        }
        _ => (),
    }

    gen(now_node.lhs.unwrap(), node_list, input);
    gen(now_node.rhs.unwrap(), node_list, input);

    println!("  pop rdi");
    println!("  pop rax");

    match now_node.kind {
        NodeKind::ADD => {
            println!("  add rax, rdi");
        }
        NodeKind::SUB => {
            println!("  sub rax, rdi");
        }
        NodeKind::MUL => {
            println!("  imul rax, rdi");
        }
        NodeKind::DIV => {
            println!("  cqo");
            println!("  idiv rdi");
        }
        NodeKind::EQ => {
            println!("  cmp rax, rdi");
            println!("  sete al");
            println!("  movzb rax, al");
        }
        NodeKind::LT => {
            println!("  cmp rax, rdi");
            println!("  setl al");
            println!("  movzb rax, al");
        }
        NodeKind::LE => {
            println!("  cmp rax, rdi");
            println!("  setle al");
            println!("  movzb rax, al");
        }
        NodeKind::NE => {
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
