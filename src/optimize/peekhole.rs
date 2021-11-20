use crate::lexer::Token;
use crate::parser::{Operation, Parser};
use crate::var_map::VariableMap;

impl Parser {
    // Return the final destination of label
    fn get_dist<'a>(
        &'a self,
        var_map: &mut VariableMap,
        from: &'a Token,
        start: &'a Token,
    ) -> &'a Token {
        let label_line = var_map.label_get(&from) as usize;
        if label_line >= self.internal_code.len() {
            return from;
        }
        let first_op = &self.internal_code[label_line];
        match first_op {
            &Operation::Goto(ref to) => {
                // If goto chains loops, return start
                if *to == *start {
                    return start;
                }
                // recurssion
                let rec = self.get_dist(var_map, to, start);
                return rec;
            }
            _ => return from,
        }
    }

    pub fn optimize_peekhole(&mut self, var_map: &mut VariableMap) {
        for i in 0..self.internal_code.len() {
            if let Operation::Goto(ref label) = self.internal_code[i] {
                let final_dist = self.get_dist(var_map, label, label);
                if final_dist != label {
                    self.internal_code[i] = Operation::Goto(final_dist.clone());
                }
            }
            if let Operation::IfGoto(ref cond, ref label) = self.internal_code[i] {
                let final_dist = self.get_dist(var_map, label, label);
                if final_dist != label {
                    self.internal_code[i] = Operation::IfGoto(cond.clone(), final_dist.clone());
                }
            }
        }
    }
}
