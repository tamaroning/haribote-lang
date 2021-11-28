mod error;
mod exec;
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

const VERSION_STR: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug)]
pub struct Options {
    emit_ir: bool,  /* whether a IR is printed or not */
    exec: bool,     /* whether a program is executed or not */
    optimize: bool, /* whether optimizer is enabled or not */
}

impl Options {
    fn new() -> Self {
        Options {
            emit_ir: false,
            exec: true,
            optimize: true,
        }
    }
}

pub fn run(s: String, opts: &Options, var_map: &mut VariableMap) {
    let mut parser = Parser::new(s);
    if let Err(e) = parser.compile(var_map) {
        println!("{}", e);
        return;
    }
    if opts.emit_ir && opts.optimize {
        println!("Optimizing...");
    }
    if opts.optimize {
        parser.optimize_constant_folding(var_map);
        parser.optimize_peekhole(var_map);
        parser.remove_unreachable_ops(var_map);
    }
    if opts.emit_ir {
        println!("--------------- Dump of internal code ---------------");
        parser.dump_internal_code(var_map);
        println!("-----------------------------------------------------");
    }
    if opts.exec {
        parser.exec(var_map);
    }
}

fn load_text(path: &str) -> String {
    let mut file = File::open(path).expect("File not found");
    let mut txt = String::new();
    file.read_to_string(&mut txt)
        .expect("Couldn't open the file");
    txt
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut options = Options::new();
    let mut filepath = None;

    if args.len() > 1 && &args[1] == "help" {
        println!("haribote-lang version {}", VERSION_STR);
        println!("Usage:");
        println!("    hrb [OPTIONS] FILEPATH    Run the program");
        println!("    hrb [OPTIONS]             Run in interactive mode");
        println!("Options:");
        println!("    -emit-ir          Display the intermidiate representation");
        println!("    -no-optimize      Doesn't optimize the program");
        println!("    -no-exec          Doesn't execute the program");
        return;
    }

    for arg in &args[1..] {
        match arg.as_str() {
            "-emit-ir" => {
                options.emit_ir = true;
            }
            "-no-exec" => {
                options.exec = false;
            }
            "-no-optimize" => {
                options.optimize = false;
            }
            _ => {
                if arg.starts_with("-") {
                    println!("Invalid option: {}", arg);
                    return;
                }
                filepath = Some(arg);
            }
        }
    }

    let mut var = VariableMap::new();

    // run the file
    if filepath != None {
        let src = load_text(filepath.unwrap());
        run(src, &options, &mut var);
    }
    // run in interactive mode
    else {
        println!("haribote-lang version {}", VERSION_STR);
        println!("Running in Interactive mode");
        println!("Type \"run <filepath>\" to load and run the file.");
        loop {
            let mut input = String::new();
            print!(">>> ");
            io::stdout().flush().unwrap();
            io::stdin().read_line(&mut input).expect("input error");
            input = input.replace("\r", "").replace("\n", "");
            // exit
            if input.as_str() == "exit" {
                std::process::exit(0);
            }
            // run the file
            else if input.starts_with("run") {
                let filepath = &input[4..];
                let src = load_text(filepath);
                run(src, &options, &mut var);
            } else {
                run(input, &options, &mut var);
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use super::*;
    use crate::lexer::Token;
    use crate::optimize;

    #[test]
    fn test_add() {
        let src = String::from("result = 100 + 200 - 50;");
        let mut var = VariableMap::new();
        run(src, &Options::new(), &mut var);
        let result = var.get(&Token::new(String::from("result"), lexer::TokenType::Ident));
        assert_eq!(result, 250);
    }

    #[test]
    fn test_expr() {
        let src = String::from("a = 10 + 2 * 7 - 4;");
        let mut var = VariableMap::new();
        run(src, &Options::new(), &mut var);
        let result = var.get(&Token::new(String::from("a"), lexer::TokenType::Ident));
        assert_eq!(result, 20);
    }

    #[test]
    fn test_int_var() {
        let src = String::from("result = 1; result = result + result * 2; result = result + 4;");
        let mut var = VariableMap::new();
        run(src, &Options::new(), &mut var);
        let result = var.get(&Token::new(String::from("result"), lexer::TokenType::Ident));
        assert_eq!(result, 7);
    }

    #[test]
    fn test_goto() {
        let src = String::from("result = 1; goto A; B: result = result * 4; goto C; A: result = result + 2; goto B; C:");
        let mut var = VariableMap::new();
        run(src, &Options::new(), &mut var);
        let result = var.get(&Token::new(String::from("result"), lexer::TokenType::Ident));
        assert_eq!(result, 12);
    }

    #[test]
    fn test_if() {
        let src = String::from(
            "a = 2; if (a <= 2) { if (a == 1) {} else { result = 10; } } else { a = 0; }",
        );
        let mut var = VariableMap::new();
        run(src, &Options::new(), &mut var);
        let result = var.get(&Token::new(String::from("result"), lexer::TokenType::Ident));
        assert_eq!(result, 10);
    }

    #[test]
    fn test_for() {
        let src = String::from("sum = 0; i = 0; for (;i <= 10; i = i + 1) { sum = sum + i; }");
        let mut var = VariableMap::new();
        run(src, &Options::new(), &mut var);
        let sum = var.get(&Token::new(String::from("sum"), lexer::TokenType::Ident));
        assert_eq!(sum, 55);
    }

    #[test]
    fn test_array() {
        let src = String::from("let a[3]; a[1] = 1; a[2] = 2;");
        let mut var = VariableMap::new();
        run(src, &Options::new(), &mut var);
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
        let _ = parser.compile(&mut var_map);
        parser.dump_internal_code(&mut var_map);
        optimize::cfg::ic_to_cfg(&parser.internal_code, &mut var_map);
    }

    #[test]
    fn test_constant_propagation_on_acyclic_graph() {
        let src = String::from("a = 1; b = 2; c = 3; c = e;");
        let mut var_map = VariableMap::new();
        let mut parser = Parser::new(src);
        let _ = parser.compile(&mut var_map);
        let cfg = optimize::cfg::ic_to_cfg(&parser.internal_code, &mut var_map);
        let const_maps = cfg.constant_propagation();
        let mut c = HashMap::new();
        c.insert(String::from("a"), Some(1));
        assert_eq!(const_maps[0].outs, c);
        c.insert(String::from("b"), Some(2));
        assert_eq!(const_maps[1].outs, c);
        c.insert(String::from("c"), Some(3));
        assert_eq!(const_maps[2].outs, c);
        c.insert(String::from("c"), None);
        assert_eq!(const_maps[3].outs, c);
    }

    #[test]
    fn test_constant_propagation_on_cyclic_graph() {
        let src = String::from("i = 0; A: i = i + 1; goto A;");
        let mut var_map = VariableMap::new();
        let mut parser = Parser::new(src);
        let _ = parser.compile(&mut var_map);
        let cfg = optimize::cfg::ic_to_cfg(&parser.internal_code, &mut var_map);
        let const_maps = cfg.constant_propagation();
        println!("{:?}", const_maps);
        let mut c = HashMap::new();
        c.insert(String::from("i"), Some(0));
        assert_eq!(const_maps[0].outs, c);
        c.insert(String::from("i"), None);
        c.insert(String::from("_tmp0"), None);
        assert_eq!(const_maps[1].outs, c);
        assert_eq!(const_maps[2].outs, c);
        assert_eq!(const_maps[3].outs, c);
    }
}
