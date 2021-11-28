use std::collections::HashSet;

use super::cfg::ic_to_cfg;
use crate::parser::{Operation, Parser};
use crate::var_map::VariableMap;

impl Parser {
    pub fn remove_unreachable_ops(&mut self, var_map: &mut VariableMap) {
        let cfg = ic_to_cfg(&self.internal_code, var_map);
        let mut is_reachable = vec![false; self.internal_code.len() + 1];

        let mut worklist = HashSet::new();
        worklist.insert(0);

        while !worklist.is_empty() {
            let n: usize = *worklist.iter().next().unwrap();
            worklist.remove(&n);

            if !is_reachable[n] {
                is_reachable[n] = true;
                for succ in &cfg.succs[n] {
                    worklist.insert(*succ);
                }
            }
        }

        for i in 0..self.internal_code.len() {
            if !is_reachable[i] {
                self.internal_code[i] = Operation::Nop;
            }
        }
        self.remove_nop(var_map);
    }
}
