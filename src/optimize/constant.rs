use super::cfg::{self, Cfg};
use crate::lexer::{Token, TokenType};
use crate::parser::{Operation, Parser};
use crate::var_map::VariableMap;
use std::collections::{HashMap, HashSet};

// constant variable map for one node
#[derive(Debug, Clone)]
pub struct ConstMap {
    // Some(_) : constant
    // None : there are mutiple reachinig definitions
    pub outs: HashMap<String, Option<i32>>,
}

impl ConstMap {
    fn new() -> Self {
        ConstMap {
            outs: HashMap::new(),
        }
    }
}

fn is_constant(ins: &HashMap<String, Option<i32>>, tok: &Token) -> bool {
    match tok.ty {
        TokenType::NumLiteral(_) => true,
        TokenType::Ident => match ins.get(&tok.string) {
            Some(Some(_)) => true,
            Some(None) => false,
            None => false,
        },
        _ => panic!(),
    }
}

// return
// None : Not constant
// Some(Some(_)) : Constant
// Some(None) : there are multiple reaching defs
fn get_constant_var(ins: &HashMap<String, Option<i32>>, tok: &Token) -> Option<i32> {
    match tok.ty {
        TokenType::NumLiteral(n) => Some(n),
        TokenType::Ident => match ins.get(&tok.string) {
            Some(Some(n)) => Some(*n),
            _ => None,
        },
        _ => panic!(),
    }
}

impl Cfg {
    pub fn constant_propagation(&self) -> Vec<ConstMap> {
        // constant valiables information for each node(operation)
        let mut const_maps: Vec<ConstMap> = vec![ConstMap::new(); self.nodes.len()];

        let mut worklist = HashSet::new();
        for i in 0..self.nodes.len() {
            worklist.insert(i);
        }
        let mut c = 0;
        while !worklist.is_empty() {
            c = c + 1;
            //if c==30 {break;}

            let idx = *worklist.iter().next().unwrap();
            worklist.remove(&idx);

            if idx >= self.nodes.len() {
                continue;
            }
            let op = &self.nodes[idx];

            // INs = union n.out (n: predecessor of node)
            let mut ins = HashMap::new();
            for pred in &self.preds[idx] {
                for (k, v) in &const_maps[*pred].outs {
                    //println!("{:?} {:?}", k, v);
                    match ins.get(k) {
                        Some(Some(n)) => {
                            // overwrite with None
                            if *v == None {
                                ins.insert(k.clone(), None);
                            } else if *n != v.unwrap() {
                                ins.insert(k.clone(), None);
                            }
                        }
                        Some(None) => {
                            ins.insert(k.clone(), None);
                        }
                        None => {
                            ins.insert(k.clone(), *v);
                        }
                    }
                }
            }
            //println!("{}.in {:?}", idx, ins);

            // INs = f(INs)
            match op {
                // TODO: support arrays
                // x = a
                &Operation::Copy(ref dist, ref operand) => {
                    //let dist_val = get_constant_var(&ins, dist);
                    let operand_val = get_constant_var(&ins, operand);
                    // when operand is a constant
                    if is_constant(&ins, operand) {
                        ins.insert(dist.string.clone(), Some(operand_val.unwrap()));
                    }
                    // when operand is not a constant
                    else {
                        ins.insert(dist.string.clone(), None);
                    }
                }
                // binary
                &Operation::Add(ref dist, ref operand1, ref operand2)
                | &Operation::Sub(ref dist, ref operand1, ref operand2)
                | &Operation::Mul(ref dist, ref operand1, ref operand2)
                | &Operation::Div(ref dist, ref operand1, ref operand2)
                | &Operation::Eq(ref dist, ref operand1, ref operand2)
                | &Operation::Ne(ref dist, ref operand1, ref operand2)
                | &Operation::Lt(ref dist, ref operand1, ref operand2)
                | &Operation::Le(ref dist, ref operand1, ref operand2) => {
                    let operand1_val = get_constant_var(&ins, operand1);
                    let operand2_val = get_constant_var(&ins, operand2);
                    if is_constant(&ins, operand1) && is_constant(&ins, operand2) {
                        let ret = match &op {
                            Operation::Add(..) => operand1_val.unwrap() + operand2_val.unwrap(),
                            Operation::Sub(..) => operand1_val.unwrap() - operand2_val.unwrap(),
                            Operation::Mul(..) => operand1_val.unwrap() * operand2_val.unwrap(),
                            Operation::Div(..) => {
                                if operand2_val.unwrap() == 0 {
                                    panic!("Found divisionn by zero");
                                }
                                operand1_val.unwrap() / operand2_val.unwrap()
                            }
                            Operation::Eq(..) => {
                                if operand1_val.unwrap() == operand2_val.unwrap() {
                                    1
                                } else {
                                    0
                                }
                            }
                            Operation::Ne(..) => {
                                if operand1_val.unwrap() != operand2_val.unwrap() {
                                    1
                                } else {
                                    0
                                }
                            }
                            Operation::Lt(..) => {
                                if operand1_val.unwrap() < operand2_val.unwrap() {
                                    1
                                } else {
                                    0
                                }
                            }
                            Operation::Le(..) => {
                                if operand1_val.unwrap() <= operand2_val.unwrap() {
                                    1
                                } else {
                                    0
                                }
                            }
                            _ => panic!(),
                        };
                        ins.insert(dist.string.clone(), Some(ret));
                    } else {
                        ins.insert(dist.string.clone(), None);
                    }
                }
                _ => (),
            }
            //println!("{}.out↓{:?}", idx, const_maps[idx].outs);
            //println!("{}.out {:?}", idx, ins);

            // if f(INs) != OUTs then pushes all successors of the node into worklist
            if ins != const_maps[idx].outs {
                const_maps[idx].outs = ins;
                for succ in &self.succs[idx] {
                    //println!("push {}", succ);
                    worklist.insert(*succ);
                }
            }
        }
        //println!("WL {:?}", worklist);
        //println!("{:?}", const_maps);
        const_maps
    }
}

impl Parser {
    pub fn optimize_constant_folding(&mut self, var_map: &mut VariableMap) {
        let cfg = cfg::ic_to_cfg(&self.internal_code, var_map);
        let const_maps = cfg.constant_propagation();
        for i in 0..self.internal_code.len() {
            match &self.internal_code[i] {
                Operation::Copy(ref dist, ref operand) => {
                    if operand.ty == TokenType::Ident {
                        match const_maps[i].outs.get(&dist.string) {
                            Some(Some(ref n)) => {
                                self.internal_code[i] = Operation::Copy(
                                    dist.clone(),
                                    Token::new_num(*n, None),
                                );
                            }
                            _ => (),
                        }
                    }
                }
                Operation::Add(ref dist, ..)
                | Operation::Sub(ref dist, ..)
                | Operation::Mul(ref dist, ..)
                | Operation::Div(ref dist, ..)
                | Operation::Eq(ref dist, ..)
                | Operation::Ne(ref dist, ..)
                | Operation::Lt(ref dist, ..)
                | Operation::Le(ref dist, ..) => match const_maps[i].outs.get(&dist.string) {
                    Some(Some(ref n)) => {
                        self.internal_code[i] = Operation::Copy(
                            dist.clone(),
                            Token::new_num(*n, None),
                        );
                    }
                    _ => (),
                },
                _ => (),
            }
        }
    }
}
