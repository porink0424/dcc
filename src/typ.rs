use crate::{error, lexer::TokenList, parser::Type};

// typ型を格納するのに必要なバイト数を取得する
pub fn get_size(typ: Type) -> usize {
    match typ {
        Type::Int(0) | Type::Unknown => 8, // TODO: レジスタの使い分けが非常に面倒なので、int型も一旦8bytes alignmentで実装する
        Type::Int(x) if x > 0 => 8,
        _ => unreachable!(),
    }
}

// 数値の二項演算において、2つの値の型に対する結果の型を返す
pub fn binary_calc_type(typ1: Type, typ2: Type, token_list: &TokenList) -> Type {
    match (typ1, typ2) {
        (Type::Int(0), Type::Int(0))
        | (Type::Int(0), Type::Unknown)
        | (Type::Unknown, Type::Int(0)) => Type::Int(0),
        (Type::Int(x), Type::Int(0))
        | (Type::Int(0), Type::Int(x))
        | (Type::Int(x), Type::Unknown)
        | (Type::Unknown, Type::Int(x)) => Type::Int(x),
        (Type::Unknown, Type::Unknown) => Type::Unknown,
        _ => {
            error::error(
                token_list.tokens[token_list.now].input_idx,
                "不正な型で演算が行われています",
                &token_list.input,
            );
            unreachable!()
        }
    }
}

// typ1 = typ2という代入において、代入が成立するか判定する
pub fn match_assign_type(typ1: Type, typ2: Type, token_list: &TokenList) {
    match typ1 {
        Type::Int(0) => match typ2 {
            // 左辺がint型であれば、Int(0), Unknownを右辺として受け付ける
            Type::Int(0) | Type::Unknown => (),
            _ => {
                error::error(
                    token_list.tokens[token_list.now].input_idx,
                    "int型への代入が不正です",
                    &token_list.input,
                );
            }
        },
        Type::Int(x) => match typ2 {
            // 左辺がintへのポインタ型であれば、同じ型のみを右辺として受け付ける
            Type::Int(y) if x == y => (),
            _ => {
                error::error(
                    token_list.tokens[token_list.now].input_idx,
                    &format!("int{}型への代入が不正です", "*".to_string().repeat(x)),
                    &token_list.input,
                );
            }
        },
        _ => {
            error::error(
                token_list.tokens[token_list.now].input_idx,
                "代入の左辺が不正な値です",
                &token_list.input,
            );
            unreachable!()
        }
    }
}
