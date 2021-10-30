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
        self.tokens.push(Token::new(String::from(".")));
        self.tokens.push(Token::new(String::from(".")));
        self.tokens.push(Token::new(String::from(".")));
    }
}

#[derive(Debug)]
enum Operation {
    Copy(Token, Token), // to, from
    Add(Token, Token, Token), // dist, lhs, rhs
    Sub(Token, Token, Token),
    Mul(Token, Token, Token),
    Div(Token, Token, Token),
    Eq(Token, Token, Token),
    Ne(Token, Token, Token),
    Lt(Token, Token, Token),
    Le(Token, Token, Token),
    Print(Token),
    Time,
    Goto(Token),
    IfGoto(Token, Token), // cond, label
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

// TODO: this is dirty. Is it better to make an expression parser?
struct Parser {
    pos: usize,
    lexer: Lexer,
    internal_code: Vec<Operation>,
    // temporary strage of operation parameters
    // see the difinition of phrase_compare
    cur_token_param: [Option<Token>; 4],
    cur_expr_param_start_pos: [usize; 4],

    // these two are used only to parse expressions
    expr_pos: usize,
    // the number of variables which is used
    // to store temporay results of culculation
    temp_var_cnt: usize,
}

macro_rules! parse_binary_op {
    ($func_name:ident, $child:ident, $op1:expr, $path1:path, $op2:expr, $path2:path) => {
        fn $func_name(&mut self) -> Token {
            let mut ret = self.$child();
            while self.expr_pos < self.lexer.tokens.len() {
                if self.lexer.tokens[self.expr_pos].matches($op1) {
                    self.expr_pos += 1;
                    let $child = self.$child();
                    let tmp = self.make_temp_var();
                    let op = $path1(tmp.clone(), ret, $child);
                    self.push_internal_code(op);
                    ret = tmp;
                } else if self.lexer.tokens[self.expr_pos].matches($op2) {
                    self.expr_pos += 1;
                    let $child = self.$child();
                    let tmp = self.make_temp_var();
                    let op = $path2(tmp.clone(), ret, $child);
                    self.push_internal_code(op);
                    ret = tmp;
                } else {
                    break;
                }
            }
            ret
        }
    };
}

impl Parser {
    fn new(s: String) -> Self {
        let mut lexer = Lexer::new(s);
        lexer.lex();
        Parser {
            pos: 0,
            lexer: lexer,
            internal_code: Vec::new(),
            cur_token_param: Default::default(),
            cur_expr_param_start_pos: [0; 4],
            expr_pos: 0,
            temp_var_cnt: 0,
        }
    }

    fn make_temp_var(&mut self) -> Token {
        let ret = Token::new(String::from(format!("_tmp{}", self.temp_var_cnt)));
        self.temp_var_cnt += 1;
        ret
    }

    fn primary(&mut self) -> Token {
        // ( expr )
        if self.lexer.tokens[self.expr_pos].matches("(") {
            self.expr_pos += 1; // "("
            let ret = self.expr();
            if !self.lexer.tokens[self.expr_pos].matches(")") {
                panic!("Missing parentheses");
            }
            self.expr_pos += 1; // ")"
            return ret;
        }
        // ident | num
        let ret = self.lexer.tokens[self.expr_pos].clone();
        self.expr_pos += 1;
        ret
    }

    fn unary(&mut self) -> Token {
        if self.lexer.tokens[self.expr_pos].matches("-") {
            self.expr_pos += 1;
            let tmp = self.make_temp_var();
            let op = Operation::Sub(tmp.clone(), Token::new(String::from("0")), self.primary());
            self.push_internal_code(op);
            return tmp;
        } else if self.lexer.tokens[self.expr_pos].matches("+") {
            self.expr_pos += 1;
            return self.primary();
        }
        return self.primary();
    }

    parse_binary_op!(mul, unary, "*", Operation::Mul, "/", Operation::Div);
    parse_binary_op!(add, mul, "+", Operation::Add, "-", Operation::Sub);
    parse_binary_op!(relational, add, "<", Operation::Lt, "<=", Operation::Le);
    parse_binary_op!(equality, relational, "==", Operation::Eq, "!=", Operation::Ne);

    // parse an expression, the begging expression of which is self.expr_pos
    fn expr(&mut self) -> Token {
        self.equality()
    }

    fn get_expr_param(&mut self, idx: usize) -> Token {
        self.temp_var_cnt = 0;
        self.expr_pos = self.cur_expr_param_start_pos[idx];
        self.expr()
    }

    fn push_internal_code(&mut self, op: Operation) {
        self.internal_code.push(op);
    }

    fn expr_len(&self, mut start_pos: usize) -> usize {
        let mut len = 0;
        match self.lexer.tokens[start_pos].string.as_str() {
            "(" => {
                start_pos += 1;
                len += 1;
                let inside_len = self.expr_len(start_pos);
                start_pos += inside_len;
                len += inside_len;
                if !self.lexer.tokens[start_pos].matches(")") {
                    panic!("Missing parentheses");
                }
                start_pos += 1;
                len += 1;
                return len;
            }
            _ => {
                //numerical literals or variables
                start_pos += 1;
                len += 1;
                while start_pos < self.lexer.tokens.len() {
                    if let "+" | "-" | "*" | "/" | "==" | "!=" | "<" | "<=" = self.lexer.tokens[start_pos].string.as_str() {
                        start_pos += 1;
                        len += 1;
                        let rhs_len = self.expr_len(start_pos);
                        start_pos += rhs_len;
                        len += rhs_len;
                    } else {
                        break;
                    }
                }
                return len;
            }
            
        }
    }

    // This function set self.cur_token_param_start_pos, and add up self.pos
    // Before call this function, make sure that self.cur_inst_len=0 and that tokens[self.pos] matches the beginning of the phrase
    // When it satisfies, tokens[self.pos + self.cur_inst_len] essentially points to the beginning of the *tXX or *eXX
    fn phrase_compare<const N: usize>(&mut self, phr: [&'static str; N]) -> bool {
        let inst_start_pos = self.pos;
        //println!("phr: {:?}", phr);
        for i in 0..N {
            //println!("compare {:?} with {:?}", self.lexer.tokens[self.pos].string, phr[i]);
            if phr[i].starts_with("*t") {
                let n = phr[i][2..].parse::<usize>().unwrap();
                // TODO: this clone can be replaced something like Option::take?
                self.cur_token_param[n] = Some(self.lexer.tokens[self.pos].clone());
            } else if phr[i].starts_with("*e") {
                let n = phr[i][2..].parse::<usize>().unwrap();
                self.cur_expr_param_start_pos[n] = self.pos;
                let expr_len = self.expr_len(self.pos);
                self.pos += expr_len;
                continue;
            } else if !self.lexer.tokens[self.pos].matches(phr[i]) {
                // unwind
                self.pos = inst_start_pos;
                return false;
            }
            self.pos += 1;
        }
        return true;
    }

    fn compile(&mut self, var: &mut VariableMap) {
        while self.pos < self.lexer.tokens.len() - 3 {
            println!(
                "instruction starts with tokens[{}]={:?}",
                self.pos, self.lexer.tokens[self.pos]
            );
            // (simple) assignment
            if self.phrase_compare(["*t0", "=", "*t1", ";"]) {
                let param0 = self.cur_token_param[0].take().unwrap();
                let param1 = self.cur_token_param[1].take().unwrap();
                self.push_internal_code(Operation::Copy(param0, param1));
            }
            // add
            else if self.phrase_compare(["*t0", "=", "*t1", "+", "*t2", ";"]) {
                let param0 = self.cur_token_param[0].take().unwrap();
                let param1 = self.cur_token_param[1].take().unwrap();
                let param2 = self.cur_token_param[2].take().unwrap();
                self.push_internal_code(Operation::Add(param0, param1, param2));
            }
            // subtract
            else if self.phrase_compare(["*t0", "=", "*t1", "-", "*t2", ";"]) {
                let param0 = self.cur_token_param[0].take().unwrap();
                let param1 = self.cur_token_param[1].take().unwrap();
                let param2 = self.cur_token_param[2].take().unwrap();
                self.push_internal_code(Operation::Sub(param0, param1, param2));
            }
            // (complicated) assignment (This can interpret the first three syntax)
            else if self.phrase_compare(["*t0", "=", "*e0", ";"]) {
                let param0 = self.cur_token_param[0].take().unwrap();
                let expr0 = self.get_expr_param(0);
                self.push_internal_code(Operation::Copy(param0, expr0));
            }
            // print
            else if self.phrase_compare(["print", "*e0", ";"]) {
                let expr0 = self.get_expr_param(0);
                let expr_param0 = Operation::Print(expr0);
                self.push_internal_code(expr_param0);
            }
            // label
            else if self.phrase_compare(["*t0", ":"]) {
                let label = self.cur_token_param[0].take().unwrap();
                var.set(&label, self.internal_code.len() as i32);
                continue;
            }
            // goto
            else if self.phrase_compare(["goto", "*t0", ";"]) {
                let param0 = self.cur_token_param[0].take().unwrap();
                self.push_internal_code(Operation::Goto(param0));
            }
            // if ( e0 ) goto label;
            else if self.phrase_compare(["if", "(", "*e0", ")", "goto", "*t0", ";"])
            {
                let label = self.cur_token_param[0].take().unwrap();
                let expr0 = self.get_expr_param(0);
                self.push_internal_code(Operation::IfGoto(expr0, label));
            }
            // time
            else if self.phrase_compare(["time", ";"]) {
                self.push_internal_code(Operation::Time);
            } else if self.lexer.tokens[self.pos].matches(";") {
                continue;
            }
            // syntax error
            else {
                panic!(
                    "Syntax error: {} {} {}",
                    self.lexer.tokens[self.pos].string,
                    self.lexer.tokens[self.pos + 1].string,
                    self.lexer.tokens[self.pos + 2].string,
                );
            }
        }
    }

    fn exec(&self, var_map: &mut VariableMap) {
        for ic in &self.internal_code {
            println!("{:?}", ic);
        }
        let t0 = unsafe { ffi::clock() };

        let mut pc = 0;
        while pc < self.internal_code.len() {
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
                    var_map.set(dist, lhs_val - rhs_val);
                }
                Operation::Mul(ref dist, ref lhs, ref rhs) => {
                    let lhs_val = var_map.get(lhs);
                    let rhs_val = var_map.get(rhs);
                    var_map.set(dist, lhs_val * rhs_val);
                }
                Operation::Div(ref dist, ref lhs, ref rhs) => {
                    let lhs_val = var_map.get(lhs);
                    let rhs_val = var_map.get(rhs);
                    if rhs_val == 0 {
                        panic!("Zero division error");
                    }
                    var_map.set(dist, lhs_val / rhs_val);
                }
                Operation::Eq(ref dist, ref lhs, ref rhs) => {
                    let lhs_val = var_map.get(lhs);
                    let rhs_val = var_map.get(rhs);
                    var_map.set(dist, if lhs_val == rhs_val {1} else {0});
                }
                Operation::Ne(ref dist, ref lhs, ref rhs) => {
                    let lhs_val = var_map.get(lhs);
                    let rhs_val = var_map.get(rhs);
                    var_map.set(dist, if lhs_val != rhs_val {1} else {0});
                }
                Operation::Lt(ref dist, ref lhs, ref rhs) => {
                    let lhs_val = var_map.get(lhs);
                    let rhs_val = var_map.get(rhs);
                    var_map.set(dist, if lhs_val < rhs_val {1} else {0});
                }
                Operation::Le(ref dist, ref lhs, ref rhs) => {
                    let lhs_val = var_map.get(lhs);
                    let rhs_val = var_map.get(rhs);
                    var_map.set(dist, if lhs_val <= rhs_val {1} else {0});
                }
                Operation::Print(ref var) => {
                    let val = var_map.get(var);
                    println!("{}", val);
                }
                Operation::Goto(ref label) => {
                    pc = var_map.get(label) as usize;
                    continue;
                }
                Operation::IfGoto(ref cond, ref label) => {
                    let cond_val = var_map.get(cond);
                    if cond_val != 0 {
                        pc = var_map.get(label) as usize;
                        continue;
                    }
                }
                Operation::Time => unsafe {
                    println!("time: {}", ffi::clock() - t0);
                },
            }
            pc += 1;
        }
    }
}

fn run(s: String, var_map: &mut VariableMap) {
    let mut parser = Parser::new(s);
    parser.compile(var_map);
    parser.exec(var_map);
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
    use super::*;

    #[test]
    fn test_numerical_literals() {
        let mut var = VariableMap::new();
        assert_eq!(var.get(&Token::new(String::from("100"))), 100);
        assert_eq!(var.get(&Token::new(String::from("+0"))), 0);
        assert_eq!(var.get(&Token::new(String::from("-30"))), -30);
    }

    #[test]
    fn test_lexer() {
        let src = String::from("v200 = 200; if(v200 / 4 == 900) goto end;");
        let mut lexer = Lexer::new(src);
        lexer.lex();
        let mut tok_strs = Vec::new();
        for tok in lexer.tokens {
            tok_strs.push(tok.string);
        }
        assert_eq!(tok_strs, vec!["v200", "=", "200", ";", "if", "(", "v200", "/", "4", "==", "900", ")", "goto" , "end", ";", ".", ".", "."]);
    }

    #[test]
    fn test_add() {
        let src = String::from("result = 100 + 200 - 50;");
        let mut var = VariableMap::new();
        run(src, &mut var);
        let result = var.get(&Token::new(String::from("result")));
        assert_eq!(result, 250);
    }
}
