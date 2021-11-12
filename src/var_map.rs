use crate::lexer::Token;
use std::collections::HashMap;

#[derive(Debug)]
pub struct VariableMap {
    // integer variables.
    // Also used for branch labels.
    // map["label"] represents the number of the line immidiately following label:
    pub map: HashMap<String, i32>,
    // integer arrays
    array_map: HashMap<String, Vec<i32>>,
}

impl VariableMap {
    pub fn new() -> Self {
        VariableMap {
            map: HashMap::new(),
            array_map: HashMap::new(),
        }
    }

    // TODO: to_string() is a bottleneck
    pub fn get(&mut self, tok: &Token) -> i32 {
        if self.map.contains_key(&tok.string) {
            return *self.map.get(&tok.string).unwrap();
        } else {
            let opt = tok.string.parse::<i32>();
            match opt {
                // numerical literals
                Ok(n) => {
                    self.map.insert(tok.string.to_string(), n);
                    n
                }
                // undeclared valriables
                Err(_) => {
                    self.map.insert(tok.string.to_string(), 0);
                    0
                }
            }
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
            .unwrap_or_else(|| panic!("Undeclared array: {}", ident.string));
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
            .unwrap_or_else(|| panic!("Undeclared array: {}", ident.string));
        if index >= arr.len() {
            panic!(
                "Index out of bounds: the len of {} is {} but the index is {}",
                ident.string,
                arr.len(),
                index
            );
        }
        arr[index] = val;
        self.array_map.insert(ident.string.to_string(), arr);
    }
}

#[test]
fn test_numerical_literals() {
    let mut var = VariableMap::new();
    assert_eq!(var.get(&Token::new(String::from("100"))), 100);
    assert_eq!(var.get(&Token::new(String::from("+0"))), 0);
    assert_eq!(var.get(&Token::new(String::from("-30"))), -30);
}
