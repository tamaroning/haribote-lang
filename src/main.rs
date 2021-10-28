extern crate libc;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::str;

mod ffi {
    extern "C" {
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
    file.read_to_string(&mut txt)
        .expect("Couldn't open the file");
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
        '=' | '+' | '-' | '*' | '/' | '!' | '%' | '&' | '~' | '|' | '<' | '>' | '?' | ':' | '.'
        | '#' => true,
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
            self.tokens.push(Token::new(s));
        }
        self.tokens.push(Token::new(String::from(";")));
        self.tokens.push(Token::new(String::from(".")));
        self.tokens.push(Token::new(String::from(".")));
        self.tokens.push(Token::new(String::from(".")));
    }
}

#[derive(Debug)]
enum Operation {
    Copy(Token, Token),
    Add(Token, Token, Token),
    Sub(Token, Token, Token),
    Print(Token),
    Time,
    Goto(Token),
    Jeq(Token, Token, Token),
    Jne(Token, Token, Token),
    Jlt(Token, Token, Token),
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
    fn set(&mut self, tok: &Token, val: i32) {
        self.map.insert(tok.string.to_string(), val);
    }
}

struct Compiler {
    pos: usize,
    lexer: Lexer,
    internal_code: Vec<Operation>,
    cur_inst_param: [Option<Token>; 8],
}

impl Compiler {
    fn new(s: String) -> Self {
        let mut lexer = Lexer::new(s);
        lexer.lex();
        Compiler {
            pos: 0,
            lexer: lexer,
            internal_code: Vec::new(),
            // temporary strage of operation parameters
            // see the difinition of phrase_compare
            cur_inst_param: Default::default(),
        }
    }

    fn push_internal_code(&mut self, op: Operation) {
        self.internal_code.push(op);
    }

    fn phrase_compare<const N: usize>(&mut self, phr: [&'static str; N]) -> bool {
        for i in 0..N {
            if phr[i].starts_with("*") {
                let n = phr[i][1..].parse::<usize>().unwrap();
                self.cur_inst_param[n] = Some(self.lexer.tokens[self.pos + i].clone());
                continue;
            } else if !self.lexer.tokens[self.pos + i].matches(phr[i]) {
                return false;
            }
        }
        return true;
    }

    fn compile(&mut self, var: &mut VariableMap) {
        while self.pos < self.lexer.tokens.len() - 3 {
            // assignment
            if self.phrase_compare(["*0", "=", "*1", ";"]) {
                let param0 = self.cur_inst_param[0].take().unwrap();
                let param1 = self.cur_inst_param[1].take().unwrap();
                self.push_internal_code(Operation::Copy(param0, param1));
            }
            // add
            else if self.phrase_compare(["*0", "=", "*1", "+", "*2", ";"]) {
                let param0 = self.cur_inst_param[0].take().unwrap();
                let param1 = self.cur_inst_param[1].take().unwrap();
                let param2 = self.cur_inst_param[2].take().unwrap();
                self.push_internal_code(Operation::Add(param0, param1, param2));
            }
            // subtract
            else if self.phrase_compare(["*0", "=", "*1", "-", "*2", ";"]) {
                let param0 = self.cur_inst_param[0].take().unwrap();
                let param1 = self.cur_inst_param[1].take().unwrap();
                let param2 = self.cur_inst_param[2].take().unwrap();
                self.push_internal_code(Operation::Sub(param0, param1, param2));
            }
            // print
            else if self.phrase_compare(["print", "*0", ";"]) {
                let param0 = self.cur_inst_param[0].take().unwrap();
                self.push_internal_code(Operation::Print(param0));
            }
            // label
            else if self.phrase_compare(["*0", ":"]) {
                let label = self.cur_inst_param[0].take().unwrap();
                var.set(&label, self.internal_code.len() as i32);
                self.pos += 2;
                continue;
            }
            // goto
            else if self.phrase_compare(["goto", "*0", ";"]) {
                let param0 = self.cur_inst_param[0].take().unwrap();
                self.push_internal_code(Operation::Goto(param0));
            }
            // if (v0 op v1) goto label;
            else if self.phrase_compare(["if", "(", "*0", "*1", "*2", ")", "goto", "*3", ";"]) {
                let label = self.cur_inst_param[3].take().unwrap();
                let lhs = self.cur_inst_param[0].take().unwrap();
                let rhs = self.cur_inst_param[2].take().unwrap();
                let bin_op = &self.cur_inst_param[1].take().unwrap();
                if &bin_op.string == "==" {
                    self.push_internal_code(Operation::Jeq(lhs, rhs, label));
                } else if &bin_op.string == "!=" {
                    self.push_internal_code(Operation::Jne(lhs, rhs, label));
                } else if &bin_op.string == "<" {
                    self.push_internal_code(Operation::Jlt(lhs, rhs, label));
                }
            }
            // time
            else if self.phrase_compare(["time", ";"]) {
                self.push_internal_code(Operation::Time);
            } else if self.lexer.tokens[self.pos].matches(";") {
                self.pos += 1;
                continue;
            }
            // syntax error
            else {
                panic!(
                    "Syntax error: {} {} {}",
                    self.lexer.tokens[self.pos].string,
                    self.lexer.tokens[self.pos + 1].string,
                    self.lexer.tokens[self.pos + 2].string
                );
            }

            // read forward until bumping into ";"
            while !self.lexer.tokens[self.pos].matches(";") {
                self.pos += 1;
            }
            self.pos += 1;
        }
    }

    fn exec(&self, var_map: &mut VariableMap) {
        let t0 = unsafe { ffi::clock() };

        //println!("IC: {:?}", self.internal_code);
        let mut pc = 0;
        while pc < self.internal_code.len() {
            //println!("pos: {}, IC: {:?}", pc, self.internal_code[pc]);
            match self.internal_code[pc] {
                Operation::Copy(ref dist, ref var) => {
                    let val = var_map.get(var);
                    var_map.set(dist, val);
                }
                Operation::Add(ref dist, ref lhs, ref rhs) => {
                    let lhs_val = var_map.get(lhs);
                    let rhs_val = var_map.get(rhs);
                    var_map.set(dist, lhs_val + rhs_val);
                }
                Operation::Sub(ref dist, ref lhs, ref rhs) => {
                    let lhs_val = var_map.get(lhs);
                    let rhs_val = var_map.get(rhs);
                    var_map.set(dist, lhs_val + rhs_val);
                }
                Operation::Print(ref var) => {
                    let val = var_map.get(var);
                    println!("{}", val);
                }
                Operation::Goto(ref label) => {
                    pc = var_map.get(label) as usize;
                    continue;
                }
                Operation::Jeq(ref lhs, ref rhs, ref label) => {
                    let lhs_val = var_map.get(lhs);
                    let rhs_val = var_map.get(rhs);
                    if lhs_val == rhs_val {
                        pc = var_map.get(label) as usize;
                        continue;
                    }
                }
                Operation::Jne(ref lhs, ref rhs, ref label) => {
                    let lhs_val = var_map.get(lhs);
                    let rhs_val = var_map.get(rhs);
                    if lhs_val != rhs_val {
                        pc = var_map.get(label) as usize;
                        continue;
                    }
                }
                Operation::Jlt(ref lhs, ref rhs, ref label) => {
                    let lhs_val = var_map.get(lhs);
                    let rhs_val = var_map.get(rhs);
                    if lhs_val < rhs_val {
                        pc = var_map.get(label) as usize;
                        continue;
                    }
                }
                Operation::Time => unsafe {
                    println!("time: {}", ffi::clock() - t0);
                },
                Operation::Nop => (),
            }
            pc += 1;
        }
    }
}

fn run(s: String, var_map: &mut VariableMap) {
    let mut compiler = Compiler::new(s);
    compiler.compile(var_map);
    compiler.exec(var_map);
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
