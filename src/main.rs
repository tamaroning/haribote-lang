use std::env::set_var;
use std::fs::File;
use std::io::prelude::*;
use std::env;
use std::str;
use std::collections::HashMap;

#[derive(Debug, Hash, PartialEq, Eq)]
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
struct Lexer<'a> {
    txt: &'a str,
    pos: usize,
    tokens: Vec<Token<'a>>,
}

impl<'a> Lexer<'a> {
    fn new(prg: &'a String) -> Self {
        Lexer {
            txt: prg,
            pos: 0,
            tokens: Vec::new(),
        }
    }
 
    fn next_char(&self) -> char {
        self.txt[self.pos..].chars().next().unwrap()
    }

    fn nth_char(&self, n: usize) -> char {
        self.txt[self.pos..].chars().nth(n).unwrap()
    }

    fn lex<'b>(&mut self) {
        while self.pos < self.txt.len() {
            let start_pos = self.pos;
            if is_whitespace(self.next_char()) {
                self.pos += 1;
                continue;
            }
            if is_one_char_symbol(self.next_char()) {
                self.pos += 1;
            } else if self.next_char().is_alphanumeric() && self.pos < self.txt.len() {
                while self.next_char().is_alphanumeric() {
                    self.pos += 1;
                }
            } else if is_normal_symbol(self.next_char()) {
                while is_normal_symbol(self.next_char()) && self.pos < self.txt.len() {
                    self.pos += 1;
                }
            } else {
                println!("Syntax error : '{}'", self.next_char());
                std::process::exit(0);
            }
            let s: &'a str = &self.txt[start_pos .. self.pos];
            //println!("{:?}", s);
            self.tokens.push(Token::new(s));
        }
    }
}


struct VariableMap<'a> {
    map: HashMap<&'a Token<'a>, i32>,
}

impl<'a> VariableMap<'a> {
    fn new() -> Self {
        let mut map = HashMap::new();
        VariableMap {
            map: map,
        }
    }

    fn get(&mut self, tok: &'a Token) -> i32 {
        if self.map.contains_key(tok) {
            return *self.map.get(tok).unwrap();
        } else {
            let opt = tok.string.parse::<i32>();
            match opt {
                // numerical literals
                Ok(n) => {
                    self.map.insert(tok, n);
                    n
                },
                // undeclared valriables
                Err(_) => {
                    self.map.insert(tok, 0);
                    0
                }
            }
        }
    }

    fn set(&mut self, tok: &'a Token, val: i32) {
        self.map.insert(tok, val);
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("haribote-lang");
        println!("Usage: haribote-lang <file path>");
        std::process::exit(0);
    }

    let filepath = &args[1];
    let src = load_text(filepath);
    let mut lexer = Lexer::new(&src);
    lexer.lex();

    let mut tokens = lexer.tokens;
    tokens.push(Token::new("."));
    tokens.push(Token::new("."));
    tokens.push(Token::new("."));

    let mut var = VariableMap::new();

    let mut pc = 0;
    while pc < tokens.len() - 3 {
        if tokens[pc + 1].matches("=") && tokens[pc + 3].matches(";") {
            let val = var.get(&tokens[pc + 2]);
            var.set(&tokens[pc], val);
        } else if tokens[pc + 1].matches("=") && tokens[pc + 3].matches("+") && tokens[pc + 5].matches(";") {
            let lhs = var.get(&tokens[pc + 2]);
            let rhs = var.get(&tokens[pc + 4]);
            var.set(&tokens[pc], lhs + rhs);
        } else if tokens[pc + 1].matches("=") && tokens[pc + 3].matches("-") && tokens[pc + 5].matches(";") {
            let lhs = var.get(&tokens[pc + 2]);
            let rhs = var.get(&tokens[pc + 4]);
            var.set(&tokens[pc], lhs - rhs);
        } else if tokens[pc].matches("print") && tokens[pc + 2].matches(";") {
            println!("{}", var.get(&tokens[pc + 1]));
        } else {
            panic!("Syntax error: {} {} {}", tokens[pc].string, tokens[pc + 1].string, tokens[pc + 2].string);
        }
        while !tokens[pc].matches(";") {
            //println!("{:?}", tokens[pc]);
            pc += 1;
        }
        pc += 1;
    }
}
