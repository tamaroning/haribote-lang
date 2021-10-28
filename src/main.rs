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

#[derive(Debug, Clone)]
struct Token {
    string: String,
}

impl Token {
    fn new(s: String) -> Self {
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
    tokens: Vec<Token>,
}

impl Lexer {
    fn new(prg: String) -> Self {
        Lexer {
            txt: prg,
            pos: 0,
            tokens: Vec::new(),
        }
    }
 
    fn next_char(&self) -> char {
        self.txt[self.pos..].chars().next().unwrap()
    }

    fn lex(&mut self) {
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
            let s = self.txt[start_pos..self.pos].to_string();
            //println!("{:?}", Token::new(s));
            self.tokens.push(Token::new(s));
        }
        self.tokens.push(Token::new(String::from(";")));
        self.tokens.push(Token::new(String::from(".")));
        self.tokens.push(Token::new(String::from(".")));
        self.tokens.push(Token::new(String::from(".")));
    }
}

enum Operation {
    Copy,
    Add,
    Sub,
    Print,
    Time,
    Goto,
    Jeq,
    Jne,
    Jlt,
    Nop,
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
        if self.map.contains_key(&tok.string) {
            return *self.map.get(&tok.string).unwrap();
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

struct InternalCode {
    op: Operation,
    param: [Option<Token>; 8],
}

impl InternalCode {
    fn new() -> Self {
        InternalCode {
            op: Operation::Nop,
            param: Default::default(),
        }
    }
}

struct Compiler {
    pos: usize,
    lexer: Lexer,
    internal_code: InternalCode,
}

impl Compiler {
    fn new(s: String) -> Self {
        let mut lexer = Lexer::new(s);
        lexer.lex();
        Compiler {
            pos: 0,
            lexer: lexer,
            internal_code: InternalCode::new(),
        }
    }

    fn phrase_compare<const N: usize>(&mut self, phr: [&'static str; N]) -> bool {
        for i in 0..N {
            if phr[i].starts_with("*") {
                let n = phr[i][1..].parse::<usize>().unwrap();
                //self.internal_code.param[n] = Some(self.lexer.tokens[self.pos + i]);
                continue;
            } else if !self.lexer.tokens[self.pos + i].matches(phr[i]) {
                return false;
            }
        }
        return true;
    }

    fn compile(&mut self, var: &mut VariableMap) {
        let t0 = unsafe {
            ffi::clock()
        };
        
        println!("{:?}", self.lexer.tokens);

        // register labels
        for pc in 0..self.lexer.tokens.len() - 3 {
            if self.lexer.tokens[pc + 1].matches(":") {
                var.set(&self.lexer.tokens[pc], pc as i32 + 2);
            }
        }

        while self.pos < self.lexer.tokens.len() - 3 {
            println!("{:?}", self.lexer.tokens[self.pos]);
            // assignment
            if self.phrase_compare(["*0", "=", "*1", ";"]) {
                self.internal_code.op = Operation::Copy;
                let val = var.get(&self.lexer.tokens[self.pos + 2]);
                var.set(&self.lexer.tokens[self.pos], val);
            }
            // add
            else if self.phrase_compare(["*0", "=", "*1", "+", "*2", ";"]) {
                self.internal_code.op = Operation::Add;
                let lhs = var.get(&self.lexer.tokens[self.pos + 2]);
                let rhs = var.get(&self.lexer.tokens[self.pos + 4]);
                var.set(&self.lexer.tokens[self.pos], lhs + rhs);
            }
            // subtract
            else if self.phrase_compare(["*0", "=", "*1", "-", "*2", ";"]) {
                self.internal_code.op = Operation::Sub;
                let lhs = var.get(&self.lexer.tokens[self.pos + 2]);
                let rhs = var.get(&self.lexer.tokens[self.pos + 4]);
                var.set(&self.lexer.tokens[self.pos], lhs - rhs);
            }
            // print
            else if self.phrase_compare(["print", "*0", ";"]) {
                self.internal_code.op = Operation::Print;
                println!("{}", var.get(&self.lexer.tokens[self.pos + 1]));
            }
            // label
            else if self.phrase_compare(["*0", ":"]) {
                self.pos += 2;
                continue;
            }
            // goto
            else if self.phrase_compare(["goto", "*0", ";"]) {
                self.internal_code.op = Operation::Goto;
                self.pos = var.get(&self.lexer.tokens[self.pos + 1]) as usize;
                continue;
            }
            // if (v0 op v1) goto label;
            else if self.phrase_compare(["if", "(", "*0", "*1", "*2", ")","goto", "*3", ";"]) {
                let gpc = var.get(&self.lexer.tokens[self.pos + 7]) as usize;
                let v0 = var.get(&self.lexer.tokens[self.pos + 2]);
                let v1 = var.get(&self.lexer.tokens[self.pos + 4]);
                if self.lexer.tokens[self.pos + 3].matches("==") && v0 == v1 {
                    self.internal_code.op = Operation::Jeq;
                    self.pos = gpc;
                    continue;
                }
                if self.lexer.tokens[self.pos + 3].matches("!=") && v0 != v1 {
                    self.internal_code.op = Operation::Jne;
                    self.pos = gpc;
                    continue;
                }
                if self.lexer.tokens[self.pos + 3].matches("<") && v0 < v1 {
                    self.internal_code.op = Operation::Jlt;
                    self.pos = gpc;
                    continue;
                }
            }
            // time
            else if self.phrase_compare(["time", ";"]) {
                self.internal_code.op = Operation::Time;
                unsafe {
                    println!("time: {}", ffi::clock() - t0);
                }
            } else if self.lexer.tokens[self.pos].matches(";") {
                // do nothing
            }
            // syntax error
            else {
                panic!("Syntax error: {} {} {}", self.lexer.tokens[self.pos].string, self.lexer.tokens[self.pos + 1].string, self.lexer.tokens[self.pos + 2].string);
            }
            while !self.lexer.tokens[self.pos].matches(";") {
                //println!("{:?}", tokens[pos]);
                self.pos += 1;
            }
            //println!("{:?}", tokens[pos]);
            self.pos += 1;
        }
    }
}

fn exec() {
    todo!();
}

fn run(s: String, var_map: &mut VariableMap) {
    let mut compiler = Compiler::new(s);
    compiler.compile(var_map);
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
