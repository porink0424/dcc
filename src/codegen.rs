use crate::parser::{NodeKind, NodeList};

// NodeListからスタックマシンをemulateする形でアセンブリを出力する
pub fn gen(now: usize, node_list: &NodeList) {
    let now_node = &node_list.nodes[now];

    if now_node.kind == NodeKind::NUM {
        println!("  push {}", now_node.val.unwrap());
        return;
    }

    gen(now_node.lhs.unwrap(), node_list);
    gen(now_node.rhs.unwrap(), node_list);

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
