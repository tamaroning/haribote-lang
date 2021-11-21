use crate::error::error_exit;
use crate::lexer::{Token, TokenType};
use std::collections::HashMap;

#[derive(Debug)]
pub struct VariableMap {
    // integer variables.
    // Also used for branch labels.
    pub map: HashMap<String, i32>,
    // integer arrays
    array_map: HashMap<String, Vec<i32>>,
    // label_map["label"] represents the number of the line immidiately following label:
    pub label_map: HashMap<String, i32>,
}

impl VariableMap {
    pub fn new() -> Self {
        VariableMap {
            map: HashMap::new(),
            array_map: HashMap::new(),
            label_map: HashMap::new(),
        }
    }

    // TODO: to_string() is a bottleneck
    pub fn get(&mut self, tok: &Token) -> i32 {
        match tok.ty {
            TokenType::NumLiteral(n) => n,
            // undeclared valriables
            TokenType::Ident => match self.map.get(&tok.string) {
                Some(n) => *n,
                None => {
                    self.map.insert(tok.string.to_string(), 0);
                    0
                }
            },
            _ => panic!(),
        }
    }

    // TODO: to_string() is a bottleneck
    pub fn set(&mut self, tok: &Token, val: i32) {
        self.map.insert(tok.string.to_string(), val);
    }

    // TODO: initialize with specified value
    pub fn array_init(&mut self, ident: &Token, size: usize) {
        self.array_map.remove(&ident.string);
        self.array_map.insert(ident.string.clone(), vec![0; size]);
    }

    pub fn array_get(&mut self, ident: &Token, index: usize) -> i32 {
        let arr = self
            .array_map
            .get(&ident.string)
            .unwrap_or_else(|| error_exit(format!("Undeclared array: {}", ident.string)));
        if index >= arr.len() {
            panic!(
                "Index out of bounds: the len of {} is {} but the index is {}",
                ident.string,
                arr.len(),
                index
            );
        }
        return arr[index];
    }

    pub fn array_set(&mut self, ident: &Token, index: usize, val: i32) {
        let mut arr = self
            .array_map
            .remove(&ident.string)
            .unwrap_or_else(|| error_exit(format!("Undeclared array: {}", ident.string)));
        if index >= arr.len() {
            error_exit(format!(
                "Index out of bounds: the len of {} is {} but the index is {}",
                ident.string,
                arr.len(),
                index
            ));
        }
        arr[index] = val;
        self.array_map.insert(ident.string.to_string(), arr);
    }

    // TODO: to_string() is a bottleneck
    pub fn label_get(&mut self, tok: &Token) -> i32 {
        match self.label_map.get(&tok.string) {
            Some(line) => *line,
            None => error_exit(String::from("Undefined label")),
        }
    }

    // TODO: to_string() is a bottleneck
    pub fn label_set(&mut self, tok: &Token, val: i32) {
        self.label_map.insert(tok.string.to_string(), val);
    }
}

#[cfg(test)]
mod var_map_tests {
    use super::*;
    use crate::lexer::TokenType;

    #[test]
    fn test_numerical_literals() {
        let mut var = VariableMap::new();
        assert_eq!(var.get(&Token::new_num(100, None)), 100);
        assert_eq!(var.get(&Token::new(String::from("a"), TokenType::Ident)), 0);
    }
}
