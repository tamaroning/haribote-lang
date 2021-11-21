use std::io::{self, Write};

use crate::error::error_exit;
use crate::lexer::TokenType;
use crate::parser::{Operation, Parser};
use crate::var_map::VariableMap;

mod ffi {
    extern "C" {
        pub fn clock() -> ::libc::clock_t;
    }
}

// executer
impl Parser {
    pub fn exec(&self, var_map: &mut VariableMap) {
        let t0 = unsafe { ffi::clock() };

        let mut pc = 0;
        while pc < self.internal_code.len() {
            match self.internal_code[pc] {
                Operation::Copy(ref dist, ref var) => {
                    let val = var_map.get(var);
                    var_map.set(dist, val);
                }
                Operation::Add(ref dist, ref lhs, ref rhs) => {
                    let lhs_val = var_map.get(lhs);
                    let rhs_val = var_map.get(rhs);
                    var_map.set(dist, lhs_val + rhs_val);
                }
                Operation::Sub(ref dist, ref lhs, ref rhs) => {
                    let lhs_val = var_map.get(lhs);
                    let rhs_val = var_map.get(rhs);
                    var_map.set(dist, lhs_val - rhs_val);
                }
                Operation::Mul(ref dist, ref lhs, ref rhs) => {
                    let lhs_val = var_map.get(lhs);
                    let rhs_val = var_map.get(rhs);
                    var_map.set(dist, lhs_val * rhs_val);
                }
                Operation::Div(ref dist, ref lhs, ref rhs) => {
                    let lhs_val = var_map.get(lhs);
                    let rhs_val = var_map.get(rhs);
                    if rhs_val == 0 {
                        error_exit(String::from("Zero division error"));
                    }
                    var_map.set(dist, lhs_val / rhs_val);
                }
                Operation::Eq(ref dist, ref lhs, ref rhs) => {
                    let lhs_val = var_map.get(lhs);
                    let rhs_val = var_map.get(rhs);
                    var_map.set(dist, if lhs_val == rhs_val { 1 } else { 0 });
                }
                Operation::Ne(ref dist, ref lhs, ref rhs) => {
                    let lhs_val = var_map.get(lhs);
                    let rhs_val = var_map.get(rhs);
                    var_map.set(dist, if lhs_val != rhs_val { 1 } else { 0 });
                }
                Operation::Lt(ref dist, ref lhs, ref rhs) => {
                    let lhs_val = var_map.get(lhs);
                    let rhs_val = var_map.get(rhs);
                    var_map.set(dist, if lhs_val < rhs_val { 1 } else { 0 });
                }
                Operation::Le(ref dist, ref lhs, ref rhs) => {
                    let lhs_val = var_map.get(lhs);
                    let rhs_val = var_map.get(rhs);
                    var_map.set(dist, if lhs_val <= rhs_val { 1 } else { 0 });
                }
                Operation::Print(ref val_tok) => {
                    match &val_tok.ty {
                        TokenType::Ident | TokenType::NumLiteral(_) => {
                            let val = var_map.get(val_tok);
                            print!("{}", val);
                        }
                        TokenType::StrLiteral => {
                            print!("{}", val_tok.string);
                        }
                        _ => error_exit(format!("Cannot print {}", val_tok.string)),
                    }
                    io::stdout().flush().unwrap();
                }
                Operation::Println(ref val_tok) => match &val_tok.ty {
                    TokenType::Ident | TokenType::NumLiteral(_) => {
                        let val = var_map.get(val_tok);
                        println!("{}", val);
                    }
                    TokenType::StrLiteral => {
                        println!("{}", val_tok.string);
                    }
                    _ => error_exit(format!("Cannot print {}", val_tok.string)),
                },
                Operation::Goto(ref label) => {
                    pc = var_map.label_get(label) as usize;
                    continue;
                }
                Operation::IfGoto(ref cond, ref label) => {
                    let cond_val = var_map.get(cond);
                    if cond_val != 0 {
                        pc = var_map.label_get(label) as usize;
                        continue;
                    }
                }
                Operation::Time => unsafe {
                    println!("time: {}", ffi::clock() - t0);
                },
                Operation::ArrayNew(ref ident, ref size_tok) => {
                    let size = var_map.get(size_tok) as usize;
                    var_map.array_init(ident, size);
                }
                Operation::ArrayGet(ref dist, ref ident, ref index_tok) => {
                    let index = var_map.get(index_tok) as usize;
                    let val = var_map.array_get(ident, index);
                    var_map.set(dist, val);
                }
                Operation::ArraySet(ref ident, ref index_tok, ref val_tok) => {
                    let index = var_map.get(index_tok) as usize;
                    let val = var_map.get(val_tok);
                    var_map.array_set(ident, index, val);
                }
                Operation::Nop => (),
            }
            pc += 1;
        }
    }
}
