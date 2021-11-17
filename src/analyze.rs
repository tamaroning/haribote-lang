use crate::lexer::{Token, TokenType};
use crate::parser::Operation;
use crate::var_map::VariableMap;
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Debug)]
pub struct Cfg {
    pub succs: Vec<Vec<usize>>,
    pub preds: Vec<Vec<usize>>,
    nodes: Vec<Operation>,
    //abs_stmts: Vec<AbsStmt>,
}

// constant variable map for one node
#[derive(Debug, Clone)]
pub struct ConstMap {
    pub outs: HashMap<String, i32>,
}

impl ConstMap {
    fn new() -> Self {
        ConstMap {
            outs: HashMap::new(),
        }
    }
}

impl Cfg {
    fn new(ops: Vec<Operation>) -> Self {
        Cfg {
            succs: vec![Vec::new(); ops.len() + 1], // plus 1 in case there is the label at the end
            preds: vec![Vec::new(); ops.len() + 1],
            nodes: ops.clone(),
            //abs_stmts: Vec::new(),
        }
    }

    pub fn constant_propagation(&self) -> Vec<ConstMap> {
        // constant valiables information for each node(operation)
        let mut const_maps: Vec<ConstMap> = vec![ConstMap::new(); self.nodes.len()];

        let mut worklist = HashSet::new();
        for i in 0..self.nodes.len() {
            worklist.insert(i);
        }
        while !worklist.is_empty() {
            let idx = *worklist.iter().next().unwrap();
            worklist.remove(&idx);

            if idx >= self.nodes.len() {
                continue;
            }
            let op = &self.nodes[idx];

            // INs = union (predecessor of node)
            let mut ins = HashMap::new();
            for pred in &self.preds[idx] {
                for (k, v) in &const_maps[*pred].outs {
                    ins.insert(k.clone(), *v);
                }
            }

            println!("{}.in {:?}", idx, ins);

            // INs = f(INs)
            match op {
                // TODO: support arrays
                // x = a
                &Operation::Copy(ref dist, ref operand) => {
                    let val = match operand.ty {
                        TokenType::NumLiteral => Some(operand.string.parse::<i32>().unwrap()),
                        TokenType::Ident => {
                            if ins.contains_key(&operand.string) {
                                Some(*ins.get(&operand.string).unwrap())
                            } else {
                                None
                            }
                        }
                        _ => None,
                    };
                    if ins.contains_key(&operand.string) || operand.ty == TokenType::NumLiteral {
                        ins.insert(dist.string.clone(), val.unwrap());
                    } else {
                        ins.remove(&dist.string);
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
                    let val1 = match operand1.ty {
                        TokenType::NumLiteral => Some(operand1.string.parse::<i32>().unwrap()),
                        TokenType::Ident => {
                            if ins.contains_key(&operand1.string) {
                                Some(*ins.get(&operand1.string).unwrap())
                            } else {
                                None
                            }
                        }
                        _ => None,
                    };
                    let val2 = match operand2.ty {
                        TokenType::NumLiteral => Some(operand2.string.parse::<i32>().unwrap()),
                        TokenType::Ident => {
                            if ins.contains_key(&operand2.string) {
                                Some(*ins.get(&operand2.string).unwrap())
                            } else {
                                None
                            }
                        }
                        _ => None,
                    };
                    if val1 != None && val2 != None {
                        ins.insert(
                            dist.string.clone(),
                            match &op {
                                Operation::Add(..) => val1.unwrap() + val2.unwrap(),
                                Operation::Sub(..) => val1.unwrap() + val2.unwrap(),
                                Operation::Mul(..) => val1.unwrap() + val2.unwrap(),
                                Operation::Div(..) => {
                                    if val2.unwrap() == 0 {
                                        panic!("Found divisionn by zero");
                                    }
                                    val1.unwrap() / val2.unwrap()
                                }
                                Operation::Eq(..) => {
                                    if val1.unwrap() == val2.unwrap() {
                                        1
                                    } else {
                                        0
                                    }
                                }
                                Operation::Ne(..) => {
                                    if val1.unwrap() != val2.unwrap() {
                                        1
                                    } else {
                                        0
                                    }
                                }
                                Operation::Lt(..) => {
                                    if val1.unwrap() < val2.unwrap() {
                                        1
                                    } else {
                                        0
                                    }
                                }
                                Operation::Le(..) => {
                                    if val1.unwrap() <= val2.unwrap() {
                                        1
                                    } else {
                                        0
                                    }
                                }
                                _ => panic!(),
                            },
                        );
                    }
                }
                _ => (),
            }

            println!("{}.out {:?}", idx, ins);

            // if f(INs) != OUTs then pushes all successors of the node into worklist
            if ins != const_maps[idx].outs {
                const_maps[idx].outs = ins;
                for succ in &self.succs[idx] {
                    println!("insert {}", succ);
                    worklist.insert(*succ);
                }
            }
        }
        println!("WL {:?}", worklist);
        println!("{:?}", const_maps);
        const_maps
    }
}

/*
#[derive(Debug)]
struct AbsStmt {
    updated: Option<Token>,
    used: Vec<Token>,
}

impl AbsStmt {
    fn new(up: Option<Token>, us: Vec<Token>) -> Self {
        AbsStmt {
            updated: up,
            used: us,
        }
    }
}

fn op_to_abs_stmt(op: &Operation) -> AbsStmt {
    match op {
        Operation::Copy(ref dist, ref operand) => {
            return AbsStmt::new(Some(dist.clone()), vec![operand.clone()]);
        }
        Operation::Add(ref dist, ref operand1, ref operand2)
        | Operation::Sub(ref dist, ref operand1, ref operand2)
        | Operation::Mul(ref dist, ref operand1, ref operand2)
        | Operation::Div(ref dist, ref operand1, ref operand2)
        | Operation::Eq(ref dist, ref operand1, ref operand2)
        | Operation::Ne(ref dist, ref operand1, ref operand2)
        | Operation::Le(ref dist, ref operand1, ref operand2)
        | Operation::Lt(ref dist, ref operand1, ref operand2) => {
            return AbsStmt::new(Some(dist.clone()), vec![operand1.clone(), operand2.clone()]);
        }
        Operation::Print(ref operand) | Operation::IfGoto(ref operand, _) => {
            return AbsStmt::new(None, vec![operand.clone()]);
        }
        Operation::Goto(..) | Operation::Time => {
            return AbsStmt::new(None, vec![]);
        }
        _ => unimplemented!(),
    }
}
*/

pub fn ic_to_cfg(ops: &Vec<Operation>, var_map: &mut VariableMap) -> Cfg {
    let mut cfg = Cfg::new(ops.clone());
    for i in 0..ops.len() {
        println!("{} {:?}", i, ops[i]);
        if let Operation::Goto(ref label) = ops[i] {
            let dist = var_map.get(label) as usize;
            cfg.succs[i].push(dist);
            cfg.preds[dist].push(i);
            continue;
        } else if let Operation::IfGoto(_, ref label) = ops[i] {
            let dist = var_map.get(label) as usize;
            cfg.succs[i].push(dist);
            cfg.preds[dist].push(i);
        }
        cfg.succs[i].push(i + 1);
        cfg.preds[i + 1].push(i);
    }
    cfg
}
