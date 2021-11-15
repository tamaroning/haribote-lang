mod analyze;
mod exec;
mod experimental;
mod lexer;
mod optimize;
mod parser;
mod var_map;

extern crate libc;
use parser::Parser;
use std::env;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::str;
use var_map::VariableMap;

pub fn run(s: String, var_map: &mut VariableMap) {
    let mut parser = Parser::new(s);
    parser.compile(var_map);
    parser.dump_internal_code(var_map);
    //println!("Optimizing...");
    //parser.experimental_optimize_goto(var_map);
    //parser.experimental_optimize_constant_folding(var_map);
    parser.dump_internal_code(var_map);
    //dbg!(analyze::ic_to_cfg(&parser.internal_code, var_map));
    parser.exec(var_map);
}

fn load_text(path: &str) -> String {
    let mut file = File::open(path.clone()).expect("File not found");
    let mut txt = String::new();
    file.read_to_string(&mut txt)
        .expect("Couldn't open the file");
    txt
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 3 {
        println!("haribote-lang");
        println!("Usage: haribote-lang <file path>");
        std::process::exit(0);
    }

    let mut var = VariableMap::new();

    if args.len() == 2 {
        let filepath = &args[1];
        let src = load_text(filepath);
        run(src, &mut var);
    }

    // REPL
    if args.len() == 1 {
        println!("haribote-lang interactive mode");
        loop {
            let mut input = String::new();
            print!("> ");
            io::stdout().flush().unwrap();
            io::stdin().read_line(&mut input).expect("input error");
            input = input.replace("\r", "");
            input = input.replace("\n", "");
            // exit
            if input.as_str() == "exit" {
                std::process::exit(0);
            }
            // run the file
            else if input.starts_with("run") {
                let filepath = &input[4..];
                let src = load_text(filepath);
                run(src, &mut var);
            } else {
                run(input, &mut var);
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::collections::{HashMap, HashSet};

    use super::*;
    use crate::{analyze::ic_to_cfg, lexer::Token};

    #[test]
    fn test_add() {
        let src = String::from("result = 100 + 200 - 50;");
        let mut var = VariableMap::new();
        run(src, &mut var);
        let result = var.get(&Token::new(String::from("result"), lexer::TokenType::Ident));
        assert_eq!(result, 250);
    }

    #[test]
    fn test_expr() {
        let src = String::from("a = 10; result = tmp = a * 2; ");
        let mut var = VariableMap::new();
        run(src, &mut var);
        let result = var.get(&Token::new(String::from("result"), lexer::TokenType::Ident));
        assert_eq!(result, 20);
    }

    #[test]
    fn test_int_var() {
        let src = String::from("result = 1; result = result + result * 2; result = result + 4;");
        let mut var = VariableMap::new();
        run(src, &mut var);
        let result = var.get(&Token::new(String::from("result"), lexer::TokenType::Ident));
        assert_eq!(result, 7);
    }

    #[test]
    fn test_goto() {
        let src = String::from("result = 1; goto A; B: result = result + 4; goto C; A: result = result + 2; goto B; C:");
        let mut var = VariableMap::new();
        run(src, &mut var);
        let result = var.get(&Token::new(String::from("result"), lexer::TokenType::Ident));
        assert_eq!(result, 7);
    }

    #[test]
    fn test_if() {
        let src = String::from(
            "a = 2; if (a <= 2) { if (a == 1) {} else { result = 10; } } else { a = 0; }",
        );
        let mut var = VariableMap::new();
        run(src, &mut var);
        let result = var.get(&Token::new(String::from("result"), lexer::TokenType::Ident));
        assert_eq!(result, 10);
    }

    #[test]
    fn test_for() {
        let src = String::from("sum = 0; i = 0; for (;i <= 10; i = i + 1) { sum = sum + i; }");
        let mut var = VariableMap::new();
        run(src, &mut var);
        let sum = var.get(&Token::new(String::from("sum"), lexer::TokenType::Ident));
        assert_eq!(sum, 55);
    }

    #[test]
    fn test_array() {
        let src = String::from("let a[3]; a[1] = 1; a[2] = 2;");
        let mut var = VariableMap::new();
        run(src, &mut var);
        let a = [
            var.array_get(&Token::new("a".to_string(), lexer::TokenType::Ident), 0),
            var.array_get(&Token::new("a".to_string(), lexer::TokenType::Ident), 1),
            var.array_get(&Token::new("a".to_string(), lexer::TokenType::Ident), 2),
        ];
        assert_eq!(a, [0, 1, 2]);
    }

    #[test]
    fn test_build_cfg() {
        let src = String::from("a = 3; b = a; A: b = 100; goto A;");
        let mut var_map = VariableMap::new();
        let mut parser = Parser::new(src);
        parser.compile(&mut var_map);
        parser.dump_internal_code(&mut var_map);
        ic_to_cfg(&parser.internal_code, &mut var_map);
    }

    #[test]
    fn test_constant_propagation_on_acyclic_graph() {
        let src = String::from("a = 1; b = 2; c = 3; c = e;");
        let mut var_map = VariableMap::new();
        let mut parser = Parser::new(src);
        parser.compile(&mut var_map);
        parser.dump_internal_code(&mut var_map);
        let cfg = ic_to_cfg(&parser.internal_code, &mut var_map);
        println!("{:?}", cfg);
        let const_info = cfg.constant_propagation();
        let mut c = HashMap::new();
        c.insert(String::from("a"), 1);
        assert_eq!(const_info[0].outs, c);
        c.insert(String::from("b"), 2);
        assert_eq!(const_info[1].outs, c);
        c.insert(String::from("c"), 3);
        assert_eq!(const_info[2].outs, c);
        c.remove(&String::from("c"));
        assert_eq!(const_info[3].outs, c);
    }
}
