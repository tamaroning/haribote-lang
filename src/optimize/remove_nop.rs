use crate::parser::{Operation, Parser};
use crate::var_map::VariableMap;

impl Parser {
    pub fn remove_nop(&mut self, var_map: &mut VariableMap) {
        let mut pos: i32 = 0;
        for _ in 0..self.internal_code.len() {
            if let Operation::Nop = self.internal_code[pos as usize] {
                self.internal_code.remove(pos as usize);
                for label in var_map.label_map.clone().keys() {
                    let line = *var_map.label_map.get(label).unwrap();
                    if line > pos as i32 {
                        var_map.label_map.insert(label.clone(), line - 1);
                    }
                }
                pos -= 1;
            }
            pos += 1;
        }
    }
}
