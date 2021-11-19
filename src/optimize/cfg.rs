use crate::{parser::Operation, var_map::VariableMap};

#[derive(Debug)]
pub struct Cfg {
    pub succs: Vec<Vec<usize>>,
    pub preds: Vec<Vec<usize>>,
    pub nodes: Vec<Operation>,
    //abs_stmts: Vec<AbsStmt>,
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
}

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
