use std::collections::{HashMap, HashSet};

use crate::lexer::Token;
use crate::parser::{Operation, Parser};
use crate::var_map::VariableMap;

enum FoldCalc {
    Add,
    Sub,
    Mul,
    Div,
}

fn get_const(consts: &mut HashMap<Token, i32>, token: &Token) -> i32 {
    let opt = token.string.parse::<i32>();
    match opt {
        Ok(n) => n,
        Err(_) => *consts.get(token).unwrap(),
    }
}

// return true iff token is a numerical literal or propagated constant
fn is_const(consts: &mut HashMap<Token, i32>, token: &Token) -> bool {
    let opt = token.string.parse::<i32>();
    match opt {
        Ok(_) => true,
        Err(_) => {
            if consts.contains_key(token) {
                return true;
            } else {
                return false;
            }
        }
    }
}

// for Operaion::Copy
fn update_dist1(consts: &mut HashMap<Token, i32>, dist: &Token, param: &Token) {
    if is_const(consts, param) {
        let val = get_const(consts, param);
        consts.insert(dist.clone(), val);
    }
}

// when all params is constant, insert dist into consts
// otherwise remove dist from consts
fn update_dist2(
    consts: &mut HashMap<Token, i32>,
    dist: &Token,
    params: [&Token; 2],
    calc: FoldCalc,
) {
    if is_const(consts, params[0]) && is_const(consts, params[1]) {
        let val = match calc {
            FoldCalc::Add => get_const(consts, params[0]) + get_const(consts, params[1]),
            FoldCalc::Sub => get_const(consts, params[0]) - get_const(consts, params[1]),
            FoldCalc::Mul => get_const(consts, params[0]) * get_const(consts, params[1]),
            FoldCalc::Div => get_const(consts, params[0]) / get_const(consts, params[1]),
        };
        consts.insert(dist.clone(), val);
    }
}

impl Parser {
    // TODO: doesn't work in this case.
    // A: goto A;
    //
    // Return the final destination of label
    // Ex:
    // A:
    //     Goto(B)
    // B:
    //     Goto(C)
    // C:
    // In this case, get_dist(A) returns C.
    fn get_dist<'a>(&'a self, var_map: &mut VariableMap, label: &'a Token) -> &'a Token {
        let label_line = var_map.get(&label) as usize;
        if label_line >= self.internal_code.len() {
            return label;
        }
        let first_op = &self.internal_code[label_line];
        match first_op {
            &Operation::Goto(ref to) => {
                let rec = self.get_dist(var_map, to);
                return rec;
            }
            _ => return label,
        }
    }

    pub fn experimental_optimize_goto(&mut self, var_map: &mut VariableMap) {
        for i in 0..self.internal_code.len() {
            if let Operation::Goto(ref label) = self.internal_code[i] {
                let final_dist = self.get_dist(var_map, label);
                if final_dist != label {
                    println!(
                        "Optimize: Goto {} → Goto {}",
                        label.string, final_dist.string
                    );
                    self.internal_code[i] = Operation::Goto(final_dist.clone());
                }
            }
        }
    }

    fn referred_labels(&self, var_map: &mut VariableMap) -> Vec<HashSet<Token>> {
        let mut label_map: Vec<HashSet<Token>> = vec![HashSet::new(); self.internal_code.len() + 1];
        // collect labels reference of which exists
        for ic in &self.internal_code {
            match ic {
                Operation::Goto(ref label) | Operation::IfGoto(_, ref label) => {
                    let label_pos = var_map.get(label);
                    label_map[label_pos as usize].insert(label.clone());
                }
                _ => (),
            }
        }
        label_map
    }

    // propagate constant until bumping into goto or ifgoto, or labels
    fn constant_propagation(
        &mut self,
        label_map: &Vec<HashSet<Token>>,
        start_pos: usize,
    ) -> HashMap<Token, i32> {
        let mut consts: HashMap<Token, i32> = HashMap::new();

        for pos in start_pos..self.internal_code.len() {
            if !label_map[pos].is_empty() /* bump into labels */ && pos != start_pos
            /* but ignore labels at start_pos */
            {
                return consts;
            }
            match self.internal_code[pos] {
                Operation::Copy(ref dist, ref val) => {
                    update_dist1(&mut consts, dist, val);
                }
                Operation::Add(ref dist, ref lhs, ref rhs) => {
                    update_dist2(&mut consts, dist, [lhs, rhs], FoldCalc::Add);
                }
                Operation::Sub(ref dist, ref lhs, ref rhs) => {
                    update_dist2(&mut consts, dist, [lhs, rhs], FoldCalc::Sub);
                }
                Operation::Mul(ref dist, ref lhs, ref rhs) => {
                    update_dist2(&mut consts, dist, [lhs, rhs], FoldCalc::Mul);
                }
                Operation::Div(ref dist, ref lhs, ref rhs) => {
                    update_dist2(&mut consts, dist, [lhs, rhs], FoldCalc::Div);
                }
                Operation::Goto(_) | Operation::IfGoto(..) => {
                    return consts;
                }
                _ => (),
            }
        }
        consts
    }

    pub fn experimental_optimize_constant_folding(&mut self, var_map: &mut VariableMap) {
        let label_map = self.referred_labels(var_map);
        let mut consts = self.constant_propagation(&label_map, 0);

        let mut ics = self.internal_code.clone();
        for i in 0..ics.len() {
            if !label_map[i as usize].is_empty() {
                // do constant propagation again
                consts = self.constant_propagation(&label_map, i);
            }
            match &ics[i] {
                Operation::Copy(dist, _)
                | Operation::Add(dist, _, _)
                | Operation::Sub(dist, _, _)
                | Operation::Mul(dist, _, _)
                | Operation::Div(dist, _, _) => {
                    if is_const(&mut consts, dist) {
                        let val_string = get_const(&mut consts, dist).to_string();
                        ics[i] = Operation::Copy(dist.clone(), Token::new(val_string));
                    }
                }
                Operation::Print(val) => {
                    if is_const(&mut consts, val) {
                        let val_string = get_const(&mut consts, val).to_string();
                        ics[i] = Operation::Print(Token::new(val_string));
                    }
                }
                Operation::Goto(_) | Operation::IfGoto(..) => {
                    // do constant propagation again
                    if i + 1 < self.internal_code.len() {
                        consts = self.constant_propagation(&label_map, i + 1);
                    }
                }
                _ => (),
            }
        }
        self.internal_code = ics;
    }
}
