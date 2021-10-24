extern crate libc;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::env;
use std::str;
use std::collections::HashMap;

mod ffi {
    extern {
        pub fn clock() -> ::libc::clock_t;
    }
}

#[derive(Debug)]
struct Token<'a> {
    string: &'a str,
}

impl<'a> Token<'a> {
    fn new(s: &'a str) -> Self {
        Token { string: s }
    }

    fn matches(&self, s: &str) -> bool {
        self.string == s
    }
}

fn load_text(path: &str) -> String {
    let mut file = File::open(path.clone()).expect("File not found");
    let mut txt = String::new();
    file.read_to_string(&mut txt).expect("Couldn't open the file");
    txt
}

fn is_whitespace(c: char) -> bool {
    match c {
        ' ' | '\n' | '\t' | '\r' => true,
        _ => false,
    }
}

fn is_one_char_symbol(c: char) -> bool {
    match c {
        '(' | ')' | '{' | '}' | '[' | ']' | ';' | ',' => true,
        _ => false,
    }
}

fn is_normal_symbol(c: char) -> bool {
    match c {
        '=' | '+' | '-' | '*' | '/' | '!' | '%' | '&' 
        | '~' | '|' | '<' | '>' | '?' | ':' | '.' | '#' => true,
        _ => false,
    }
}

#[derive(Debug)]
struct Lexer {
    txt: String,
    pos: usize,
}

impl Lexer {
    fn new(prg: String) -> Self {
        Lexer {
            txt: prg,
            pos: 0,
        }
    }
 
    fn next_char(&self) -> char {
        self.txt[self.pos..].chars().next().unwrap()
    }

    fn nth_char(&self, n: usize) -> char {
        self.txt[self.pos..].chars().nth(n).unwrap()
    }

    fn lex(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        while self.pos < self.txt.len() {
            let start_pos = self.pos;
            if is_whitespace(self.next_char()) {
                self.pos += 1;
                continue;
            }
            if is_one_char_symbol(self.next_char()) {
                self.pos += 1;
            } else if self.next_char().is_alphanumeric() {
                self.pos += 1;
                while self.pos < self.txt.len() && self.next_char().is_alphanumeric() {
                    self.pos += 1;
                }
            } else if is_normal_symbol(self.next_char()) {
                self.pos += 1;
                while self.pos < self.txt.len() && is_normal_symbol(self.next_char()) {
                    self.pos += 1;
                }
            } else {
                println!("Syntax error : '{}'", self.next_char());
                std::process::exit(0);
            }
            let s: &str = &self.txt[start_pos .. self.pos];
            //println!("{:?}", Token::new(s));
            tokens.push(Token::new(s));
        }
        tokens.push(Token::new(";"));
        tokens.push(Token::new("."));
        tokens.push(Token::new("."));
        tokens.push(Token::new("."));
        tokens
    }
}

struct Parser<'a> {
    tokens: Vec<Token<'a>>,
    pos: usize,
    internal_code: [Option<&'a Token<'a>>; 8],
}

impl<'a> Parser<'a> {
    fn new(token_vec: Vec<Token<'a>>) -> Self {
        Parser {
            tokens: token_vec,
            pos: 0,
            internal_code: [None; 8],
        }
    }

    fn phrase_compare(&self, phr: Vec<&str>) -> bool {
        for i in 0..phr.len() {
            if phr[i].starts_with("*") {
                let n = phr[i][1..].parse::<usize>().unwrap();
                //INTERNAL_CODE[n] = Some(&self.tokens[self.pos + i]);
                continue;
            } else if !self.tokens[self.pos + i].matches(phr[i]) {
                return false;
            }
        }
        return true;
    }
}

struct VariableMap {
    map: HashMap<String, i32>,
}

impl VariableMap {
    fn new() -> Self {
        VariableMap {
            map: HashMap::new(),
        }
    }

    // TODO: to_string() is a bottleneck
    fn get(&mut self, tok: &Token) -> i32 {
        if self.map.contains_key(tok.string) {
            return *self.map.get(tok.string).unwrap();
        } else {
            let opt = tok.string.parse::<i32>();
            match opt {
                // numerical literals
                Ok(n) => {
                    self.map.insert(tok.string.to_string(), n);
                    n
                },
                // undeclared valriables
                Err(_) => {
                    self.map.insert(tok.string.to_string(), 0);
                    0
                }
            }
        }
    }

    // TODO: to_string() is a bottleneck
    fn set(&mut self, tok: &Token, val: i32) {
        self.map.insert(tok.string.to_string(), val);
    }
}

fn run(s: String, var: &mut VariableMap) {
    let t0 = unsafe {
        ffi::clock()
    };
    
    let mut lexer = Lexer::new(s);
    let tokens = lexer.lex();

    // register labels
    for pc in 0..tokens.len() - 3 {
        if tokens[pc + 1].matches(":") {
            var.set(&tokens[pc], pc as i32 + 2);
        }
    }

    let mut parser = Parser::new(tokens);

    while parser.pos < parser.tokens.len() - 3 {
        // assignment
        if parser.phrase_compare(vec!["*0", "=", "*1", ";"]) {
            let val = var.get(&parser.tokens[parser.pos + 2]);
            var.set(&parser.tokens[parser.pos], val);
        }
        // add
        else if parser.phrase_compare(vec!["*0", "=", "*1", "+", "*2", ";"]) {
            let lhs = var.get(&parser.tokens[parser.pos + 2]);
            let rhs = var.get(&parser.tokens[parser.pos + 4]);
            var.set(&parser.tokens[parser.pos], lhs + rhs);
        }
        // subtract
        else if parser.phrase_compare(vec!["*0", "=", "*1", "-", "*2", ";"]) {
            let lhs = var.get(&parser.tokens[parser.pos + 2]);
            let rhs = var.get(&parser.tokens[parser.pos + 4]);
            var.set(&parser.tokens[parser.pos], lhs - rhs);
        }
        // print
        else if parser.phrase_compare(vec!["print", "*0", ";"]) {
            println!("{}", var.get(&parser.tokens[parser.pos + 1]));
        }
        // label
        else if parser.phrase_compare(vec!["*0", ":"]) {
            parser.pos += 2;
            continue;
        }
        // goto
        else if parser.phrase_compare(vec!["goto", "*0", ";"]) {
            parser.pos = var.get(&parser.tokens[parser.pos + 1]) as usize;
            continue;
        }
        // if (v0 op v1) goto label;
        else if parser.phrase_compare(vec!["if", "(", "*0", "*1", "*2", ")","goto", "*3", ";"]) {
            let gpc = var.get(&parser.tokens[parser.pos + 7]) as usize;
            let v0 = var.get(&parser.tokens[parser.pos + 2]);
            let v1 = var.get(&parser.tokens[parser.pos + 4]);
            if parser.tokens[parser.pos + 3].matches("==") && v0 == v1 {
                parser.pos = gpc;
                continue;
            }
            if parser.tokens[parser.pos + 3].matches("!=") && v0 != v1 {
                parser.pos = gpc;
                continue;
            }
            if parser.tokens[parser.pos + 3].matches("<") && v0 < v1 {
                parser.pos = gpc;
                continue;
            }
        }
        // time
        else if parser.phrase_compare(vec!["time", ";"]) {
            unsafe {
                println!("time: {}", ffi::clock() - t0);
            }
        } else if parser.tokens[parser.pos].matches(";") {
            // do nothing
        }
        // syntax error
        else {
            panic!("Syntax error: {} {} {}", parser.tokens[parser.pos].string, parser.tokens[parser.pos + 1].string, parser.tokens[parser.pos + 2].string);
        }
        while !parser.tokens[parser.pos].matches(";") {
            //println!("{:?}", parser.tokens[parser.pos]);
            parser.pos += 1;
        }
        //println!("{:?}", parser.tokens[parser.pos]);
        parser.pos += 1;
    }
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
            }
            else {
                run(input, &mut var);
            }
        }
    }
}
