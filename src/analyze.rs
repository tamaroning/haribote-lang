use crate::lexer::{Token, TokenType};
use crate::parser::Operation;
use crate::var_map::VariableMap;
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Debug)]
pub struct Cfg {
    succs: Vec<Vec<usize>>,
    preds: Vec<Vec<usize>>,
    nodes: Vec<Operation>,
    //abs_stmts: Vec<AbsStmt>,
}

// constant variable list for one node
#[derive(Debug, Clone)]
pub struct ConstVarInfo {
    pub outs: HashMap<String, i32>,
}

impl ConstVarInfo {
    fn new() -> Self {
        ConstVarInfo {
            outs: HashMap::new(),
        }
    }
}

impl Cfg {
    fn new(ops: Vec<Operation>) -> Self {
        Cfg {
            succs: vec![Vec::new(); ops.len()],
            preds: vec![Vec::new(); ops.len()],
            nodes: ops.clone(),
            //abs_stmts: Vec::new(),
        }
    }

    pub fn constant_propagation(&self) -> Vec<ConstVarInfo> {
        // constant valiables information for each node(operation)
        let mut const_info: Vec<ConstVarInfo> = vec![ConstVarInfo::new(); self.nodes.len()];

        let mut worklist = HashSet::new();
        if self.nodes.len() > 0 {
            worklist.insert(0);
        }
        while !worklist.is_empty() {
            let idx = *worklist.iter().next().unwrap();
            worklist.remove(&idx);

            let op = &self.nodes[idx];

            // INs = ∩ predecessor of node
            let mut ins = HashMap::new();
            for pred in &self.preds[idx] {
                for (var, val) in &const_info[*pred].outs {
                    ins.insert(var.clone(), *val);
                }
            }
            println!("{}.in {:?}", idx, ins);

            // INs = f(INs)
            match op {
                // x = a
                &Operation::Copy(ref dist, ref operand) => {
                    // a is a constant
                    if let TokenType::NumLiteral = operand.ty {
                        ins.insert(dist.string.clone(), operand.string.parse::<i32>().unwrap());
                    } else if let TokenType::Ident = operand.ty {
                        // a is a constant
                        if ins.contains_key(&operand.string) {
                            ins.insert(dist.string.clone(), *ins.get(&operand.string).unwrap());
                        }
                        // a is not a constant
                        else {
                            ins.remove(&dist.string);
                        }
                    }
                }
                // binary
                _ => todo!(),
            }

            println!("{}.out {:?}", idx, ins);

            // if f(INs) != OUTs then pushes all successors of the node into worklist
            if ins != const_info[idx].outs {
                const_info[idx].outs = ins;
                for succ in &self.succs[idx] {
                    worklist.insert(*succ);
                }
            }
        }
        const_info
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
        if i < ops.len() - 1 {
            cfg.succs[i].push(i + 1);
            cfg.preds[i + 1].push(i);
        }
    }
    cfg
}
