extern crate libc;
use std::collections::{HashMap, HashSet};
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
            
            // string literals
            if self.next_char() == '"' {
                self.pos += 1;

                let mut dq_found = false;
                while self.pos < self.txt.len() {
                    if self.next_char() == '"' {
                        dq_found = true;
                        self.pos += 1;
                        break;
                    }
                    self.pos += 1;
                }
                if !dq_found {
                    panic!("Lexer error: Unmatched '\"'");
                }
                let mut s = self.txt[start_pos + 1..self.pos - 1].to_string();
                s = s.replace("\\n", "\n");
                self.tokens.push(Token::new(s));
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

#[derive(Debug, Clone)]
enum Operation {
    Copy(Token, Token),       // to, from
    Add(Token, Token, Token), // dist, lhs, rhs
    Sub(Token, Token, Token),
    Mul(Token, Token, Token),
    Div(Token, Token, Token),
    Eq(Token, Token, Token),
    Ne(Token, Token, Token),
    Lt(Token, Token, Token),
    Le(Token, Token, Token),
    Print(Token),
    PrintS(Token),
    Time,
    Goto(Token),
    IfGoto(Token, Token),          // cond, label
    ArrayNew(Token, Token),        // name, size
    ArraySet(Token, Token, Token), // name, index, val
    ArrayGet(Token, Token, Token), // dist, name, index
}

#[derive(Debug)]
struct VariableMap {
    map: HashMap<String, i32>,
    array_map: HashMap<String, Vec<i32>>,
}

impl VariableMap {
    fn new() -> Self {
        VariableMap {
            map: HashMap::new(),
            array_map: HashMap::new(),
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

    // TODO: initialize with specified value
    fn array_init(&mut self, ident: &Token, size: usize) {
        self.array_map.remove(&ident.string);
        self.array_map.insert(ident.string.clone(), vec![0; size]);
    }

    fn array_get(&mut self, ident: &Token, index: usize) -> i32 {
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

    fn array_set(&mut self, ident: &Token, index: usize, val: i32) {
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

#[derive(PartialEq, Eq)]
enum Block {
    IfElse(Token, Option<Token>),           // L0, L1
    For(Token, Token, Token, usize, usize), // L0, L1, L2, e1_start, e2_start
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
    temp_label_cnt: usize,
    blocks: Vec<Block>,
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
            temp_label_cnt: 0,
            blocks: Vec::new(),
        }
    }

    fn make_temp_var(&mut self) -> Token {
        let ret = Token::new(String::from(format!("_tmp{}", self.temp_var_cnt)));
        self.temp_var_cnt += 1;
        ret
    }
    fn make_temp_label(&mut self) -> Token {
        let ret = Token::new(String::from(format!("_tmpLabel{}", self.temp_label_cnt)));
        self.temp_label_cnt += 1;
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
        let ident = self.lexer.tokens[self.expr_pos].clone();
        self.expr_pos += 1;

        // ident[ expr ]
        if self.lexer.tokens[self.expr_pos].matches("[") {
            self.expr_pos += 1; // "["
            let index = self.expr();
            if !self.lexer.tokens[self.expr_pos].matches("]") {
                panic!("Unmatched parentheses");
            }
            self.expr_pos += 1; // "]"
            let ret = self.make_temp_var();
            self.push_internal_code(Operation::ArrayGet(ret.clone(), ident, index));
            return ret;
        }
        ident
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
    parse_binary_op!(
        equality,
        relational,
        "==",
        Operation::Eq,
        "!=",
        Operation::Ne
    );

    // TODO: assign to array
    fn assign(&mut self) -> Token {
        let equality = self.equality();
        if self.lexer.tokens[self.expr_pos].matches("=") {
            self.expr_pos += 1;
            let assign = self.assign();
            self.push_internal_code(Operation::Copy(equality, assign.clone()));
            return assign;
        }
        equality
    }

    // parse an expression whose starts from self.expr_pos
    fn expr(&mut self) -> Token {
        self.assign()
    }

    // TODO: Refactor
    fn get_expr_param(&mut self, idx: usize) -> Token {
        self.evaluate_expr(self.cur_expr_param_start_pos[idx])
    }

    fn get_expr_opt_param(&mut self, idx: usize) -> Option<Token> {
        self.evaluate_opt_expr(self.cur_expr_param_start_pos[idx])
    }

    // evaluate optional expression
    fn evaluate_opt_expr(&mut self, start_pos: usize) -> Option<Token> {
        if self.lexer.tokens[start_pos].matches(";") {
            return None;
        }
        self.temp_var_cnt = 0;
        self.expr_pos = start_pos;
        Some(self.expr())
    }

    fn evaluate_expr(&mut self, start_pos: usize) -> Token {
        self.temp_var_cnt = 0;
        self.expr_pos = start_pos;
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
                    panic!("Missing closing parentheses \")\"");
                }
                //start_pos += 1;
                len += 1;
                return len;
            }
            _ => {
                //numerical literals or variables
                start_pos += 1;
                len += 1;

                if self.lexer.tokens[start_pos].matches("[") {
                    start_pos += 1;
                    len += 1;
                    let inside_len = self.expr_len(start_pos);
                    start_pos += inside_len;
                    len += inside_len;
                    if !self.lexer.tokens[start_pos].matches("]") {
                        panic!("Missing closing parentheses \"]\"");
                    }
                    start_pos += 1;
                    len += 1;
                }

                while start_pos < self.lexer.tokens.len() {
                    if let "+" | "-" | "*" | "/" | "==" | "!=" | "<" | "<=" | "=" =
                        self.lexer.tokens[start_pos].string.as_str()
                    {
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
    // Wildcard:
    // *tXX: any token, *eXX: any expression (length > 0), **eXX: any expression (length >= 0. If length = 0, the beginning must be ";" or ")")
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
            } else if phr[i].starts_with("**e") {
                let n = phr[i][3..].parse::<usize>().unwrap();
                self.cur_expr_param_start_pos[n] = self.pos;
                if !self.lexer.tokens[self.pos].matches(";")
                    && !self.lexer.tokens[self.pos].matches(")")
                {
                    let expr_len = self.expr_len(self.pos);
                    self.pos += expr_len;
                }
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
            // println!("Statement starts with tokens[{}]={:?}", self.pos, self.lexer.tokens[self.pos]);
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
                self.
                push_internal_code(Operation::Print(expr0));
            }
            else if self.phrase_compare(["prints", "*t0", ";"]) {
                let s = self.cur_token_param[0].take().unwrap();
                self.push_internal_code(Operation::PrintS(s));
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
            else if self.phrase_compare(["if", "(", "*e0", ")", "goto", "*t0", ";"]) {
                let label = self.cur_token_param[0].take().unwrap();
                let expr0 = self.get_expr_param(0);
                self.push_internal_code(Operation::IfGoto(expr0, label));
            }
            // Parsing "if" statement
            // if (*e0) {
            //     A
            // }
            // ↓
            // IfGoto(!e0, L0)
            // A
            // L0:
            //
            // if (e0) {
            //     A
            // } else {
            //     B
            // }
            // ↓
            // IfGoto(!e0, L0)
            // A
            // Goto(L1)
            // L0:
            // B
            // L1:
            else if self.phrase_compare(["if", "(", "*e0", ")", "{"]) {
                let label0 = self.make_temp_label();
                self.blocks.push(Block::IfElse(label0.clone(), None)); // push L0

                let expr0 = self.get_expr_param(0);
                let not_expr0 = self.make_temp_var();
                self.push_internal_code(Operation::Eq(
                    not_expr0.clone(),
                    expr0,
                    Token::new(String::from("0")),
                ));
                self.push_internal_code(Operation::IfGoto(not_expr0, label0.clone()));
            // if (!e0) goto L0;
            } else if self.phrase_compare(["}", "else", "{"]) {
                let block = self
                    .blocks
                    .pop()
                    .unwrap_or_else(|| panic!("Unmatced braces"));
                let label0 = match block {
                    Block::IfElse(ref label0, None) => label0,
                    _ => panic!("Unmatched else"),
                };
                let label1 = self.make_temp_label();
                self.blocks
                    .push(Block::IfElse(label0.clone(), Some(label1.clone())));

                self.push_internal_code(Operation::Goto(label1)); // Goto(L1)
                var.set(label0, self.internal_code.len() as i32); // L0:
            }
            // Parsing for statement
            // for (**e0; **e1; **e2) {
            //     A
            // }
            // ↓
            // evaluate e0 (output if e0 exists)
            // IfGoto(!e1, L0) (output if e1 exists)
            // L1:
            // A
            // L2: (this label is referred by "continue")
            // evaluate e2 (output if e2 exists)
            // IfGoto(e1, L1) (Goto(L1) is output if e1 dosen't exist)
            // L0:
            else if self.phrase_compare(["for", "(", "**e0", ";", "**e1", ";", "**e2", ")", "{"])
            {
                let label0 = self.make_temp_label();
                let label1 = self.make_temp_label();
                let label2 = self.make_temp_label();
                self.blocks.push(Block::For(
                    label0.clone(),
                    label1.clone(),
                    label2,
                    self.cur_expr_param_start_pos[1],
                    self.cur_expr_param_start_pos[2],
                ));

                self.get_expr_opt_param(0); // evaluate e0
                let opt_expr1 = self.get_expr_opt_param(1);
                if let Some(expr1) = opt_expr1 {
                    let not_expr1 = self.make_temp_var();
                    self.push_internal_code(Operation::Eq(
                        not_expr1.clone(),
                        expr1,
                        Token::new(String::from("0")),
                    ));
                    self.push_internal_code(Operation::IfGoto(not_expr1, label0.clone()));
                    // if (!e0) goto L0;
                }
                var.set(&label1, self.internal_code.len() as i32); // L1:
            } else if self.phrase_compare(["}"]) {
                let block = self
                    .blocks
                    .pop()
                    .unwrap_or_else(|| panic!("Unmatched braces"));
                match block {
                    Block::IfElse(ref label0, None) => {
                        var.set(label0, self.internal_code.len() as i32); // L0:
                    }
                    Block::IfElse(_, Some(ref label1)) => {
                        var.set(label1, self.internal_code.len() as i32); // L1:
                    }
                    Block::For(ref label0, label1, ref label2, e1_start_pos, e2_start_pos) => {
                        var.set(label2, self.internal_code.len() as i32); // L2:
                        self.evaluate_opt_expr(e2_start_pos);
                        let opt_expr1 = self.evaluate_opt_expr(e1_start_pos);
                        // if e1 (conditions) exists, emits IfGoto otherwise emits Goto
                        match opt_expr1 {
                            Some(expr1) => {
                                self.push_internal_code(Operation::IfGoto(expr1, label1));
                            }
                            _ => {
                                self.push_internal_code(Operation::Goto(label1));
                            }
                        }
                        var.set(label0, self.internal_code.len() as i32); // L0:
                    }
                }
            }
            // time
            else if self.phrase_compare(["time", ";"]) {
                self.push_internal_code(Operation::Time);
            } else if self.phrase_compare(["let", "*t0", "[", "*e0", "]", ";"]) {
                let param0 = self.cur_token_param[0].take().unwrap();
                let expr0 = self.get_expr_param(0);
                self.push_internal_code(Operation::ArrayNew(param0, expr0));
            }
            // assign to array elements (evaluate_expr support array with read-only)
            else if self.phrase_compare(["*t0", "[", "*e0", "]", "=", "*e1", ";"]) {
                let param0 = self.cur_token_param[0].take().unwrap();
                // Don't reset temp_var_cnt, or it may be the case that _tmp0 is used by e0 and _tmp0 is used by e1.
                self.temp_var_cnt = 0;
                self.expr_pos = self.cur_expr_param_start_pos[0];
                let expr0 = self.expr();
                self.expr_pos = self.cur_expr_param_start_pos[1];
                let expr1 = self.expr();

                self.push_internal_code(Operation::ArraySet(param0, expr0, expr1));
            } else if self.phrase_compare(["*e0", ";"]) {
                self.get_expr_param(0);
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

    fn dump_internal_code(&self, var_map: &mut VariableMap) {
        let mut label_map: Vec<HashSet<Token>> = vec![HashSet::new(); self.internal_code.len() + 1];

        // show the all labels
        for s in var_map.map.keys() {
            let line = var_map.map.get(s).unwrap();
            label_map[*line as usize].insert(Token::new(s.clone()));
        }

        /*
        // show the labels reference of which exists
        for ic in &self.internal_code {
            match ic {
                Operation::Goto(ref label) | Operation::IfGoto(_, ref label) => {
                    let label_pos = var_map.get(label);
                    label_map[label_pos as usize].insert(label.clone());
                }
                _ => (),
            }
        }
        */
        println!("--------------- Dump of internal code ---------------");
        for i in 0..=self.internal_code.len() {
            for label in &label_map[i] {
                println!("{}:", label.string);
            }
            if i != self.internal_code.len() {
                println!("    {:?}", self.internal_code[i]);
            }
        }
        println!("-----------------------------------------------------");
    }
}

// executer
impl Parser {
    fn exec(&self, var_map: &mut VariableMap) {
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
                    var_map.set(dist, if lhs_val == rhs_val { 1 } else { 0 });
                }
                Operation::Ne(ref dist, ref lhs, ref rhs) => {
                    let lhs_val = var_map.get(lhs);
                    let rhs_val = var_map.get(rhs);
                    var_map.set(dist, if lhs_val != rhs_val { 1 } else { 0 });
                }
                Operation::Lt(ref dist, ref lhs, ref rhs) => {
                    let lhs_val = var_map.get(lhs);
                    let rhs_val = var_map.get(rhs);
                    var_map.set(dist, if lhs_val < rhs_val { 1 } else { 0 });
                }
                Operation::Le(ref dist, ref lhs, ref rhs) => {
                    let lhs_val = var_map.get(lhs);
                    let rhs_val = var_map.get(rhs);
                    var_map.set(dist, if lhs_val <= rhs_val { 1 } else { 0 });
                }
                Operation::Print(ref var) => {
                    let val = var_map.get(var);
                    println!("{}", val);
                }
                Operation::PrintS(ref tok) => {
                    print!("{}", tok.string);
                    io::stdout().flush().unwrap();
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
                Operation::ArrayNew(ref ident, ref size_tok) => {
                    let size = var_map.get(size_tok) as usize;
                    var_map.array_init(ident, size);
                }
                Operation::ArrayGet(ref dist, ref ident, ref index_tok) => {
                    let index = var_map.get(index_tok) as usize;
                    let val = var_map.array_get(ident, index);
                    var_map.set(dist, val);
                }
                Operation::ArraySet(ref ident, ref index_tok, ref val_tok) => {
                    let index = var_map.get(index_tok) as usize;
                    let val = var_map.get(val_tok);
                    var_map.array_set(ident, index, val);
                }
            }
            pc += 1;
        }
    }
}

fn run(s: String, var_map: &mut VariableMap) {
    let mut parser = Parser::new(s);
    parser.compile(var_map);
    parser.dump_internal_code(var_map);
    println!("Optimizing...");
    parser.optimize_goto(var_map);
    parser.optimize_constant_folding(var_map);
    parser.dump_internal_code(var_map);
    parser.exec(var_map);
}

enum FoldCulc {
    Add,
    Sub,
    Mul,
    Div,
}

fn get_const(consts: &mut HashMap<Token, i32>, token: &Token) -> i32 {
    let opt = token.string.parse::<i32>();
    match opt {
        Ok(n) => n,
        Err(_) => *consts.get(token).unwrap(),
    }
}

// return true iff token is a numerical literal or propagated constant
fn is_const(consts: &mut HashMap<Token, i32>, token: &Token) -> bool {
    let opt = token.string.parse::<i32>();
    match opt {
        Ok(_) => true,
        Err(_) => {
            if consts.contains_key(token) {
                return true;
            } else {
                return false;
            }
        }
    }
}

// for Operaion::Copy
fn update_dist1(consts: &mut HashMap<Token, i32>, dist: &Token, param: &Token) {
    if is_const(consts, param) {
        let val = get_const(consts, param);
        consts.insert(dist.clone(), val);
    }
}

// when all params is constant, insert dist into consts
// otherwise remove dist from consts
fn update_dist2(
    consts: &mut HashMap<Token, i32>,
    dist: &Token,
    params: [&Token; 2],
    culc: FoldCulc,
) {
    if is_const(consts, params[0]) && is_const(consts, params[1]) {
        let val = match culc {
            FoldCulc::Add => get_const(consts, params[0]) + get_const(consts, params[1]),
            FoldCulc::Sub => get_const(consts, params[0]) - get_const(consts, params[1]),
            FoldCulc::Mul => get_const(consts, params[0]) * get_const(consts, params[1]),
            FoldCulc::Div => get_const(consts, params[0]) / get_const(consts, params[1]),
        };
        consts.insert(dist.clone(), val);
    }
}

impl Parser {
    // Return the final destination of label
    // Ex:
    // A:
    //     Goto(B)
    // B:
    //     Goto(C)
    // C:
    // In this case, get_dist(A) returns C.
    fn get_dist<'a>(&'a self, var_map: &mut VariableMap, label: &'a Token) -> &'a Token {
        let label_line = var_map.get(&label) as usize;
        if label_line >= self.internal_code.len() {
            return label;
        }
        let first_op = &self.internal_code[label_line];
        match first_op {
            &Operation::Goto(ref to) => {
                let rec = self.get_dist(var_map, to);
                return rec;
            }
            _ => return label,
        }
    }

    fn optimize_goto(&mut self, var_map: &mut VariableMap) {
        for i in 0..self.internal_code.len() {
            if let Operation::Goto(ref label) = self.internal_code[i] {
                let final_dist = self.get_dist(var_map, label);
                if final_dist != label {
                    println!(
                        "Optimize: Goto {} → Goto {}",
                        label.string, final_dist.string
                    );
                    self.internal_code[i] = Operation::Goto(final_dist.clone());
                }
            }
        }
    }

    fn referred_labels(&self, var_map: &mut VariableMap) -> Vec<HashSet<Token>> {
        let mut label_map: Vec<HashSet<Token>> = vec![HashSet::new(); self.internal_code.len() + 1];
        // collect labels reference of which exists
        for ic in &self.internal_code {
            match ic {
                Operation::Goto(ref label) | Operation::IfGoto(_, ref label) => {
                    let label_pos = var_map.get(label);
                    label_map[label_pos as usize].insert(label.clone());
                }
                _ => (),
            }
        }
        label_map
    }

    // propagate constant until bumping into goto or ifgoto, or labels
    fn constant_propagation(
        &mut self,
        label_map: &Vec<HashSet<Token>>,
        start_pos: usize,
    ) -> HashMap<Token, i32> {
        let mut consts: HashMap<Token, i32> = HashMap::new();

        for pos in start_pos..self.internal_code.len() {
            if !label_map[pos].is_empty() /* bump into labels */ && pos != start_pos
            /* but ignore labels at start_pos */
            {
                return consts;
            }
            match self.internal_code[pos] {
                Operation::Copy(ref dist, ref val) => {
                    update_dist1(&mut consts, dist, val);
                }
                Operation::Add(ref dist, ref lhs, ref rhs) => {
                    update_dist2(&mut consts, dist, [lhs, rhs], FoldCulc::Add);
                }
                Operation::Sub(ref dist, ref lhs, ref rhs) => {
                    update_dist2(&mut consts, dist, [lhs, rhs], FoldCulc::Sub);
                }
                Operation::Mul(ref dist, ref lhs, ref rhs) => {
                    update_dist2(&mut consts, dist, [lhs, rhs], FoldCulc::Mul);
                }
                Operation::Div(ref dist, ref lhs, ref rhs) => {
                    update_dist2(&mut consts, dist, [lhs, rhs], FoldCulc::Div);
                }
                Operation::Goto(_) | Operation::IfGoto(..) => {
                    return consts;
                }
                _ => (),
            }
        }
        consts
    }

    fn optimize_constant_folding(&mut self, var_map: &mut VariableMap) {
        let label_map = self.referred_labels(var_map);
        let mut consts = self.constant_propagation(&label_map, 0);

        let mut ics = self.internal_code.clone();
        for i in 0..ics.len() {
            if !label_map[i as usize].is_empty() {
                // do constant propagation again
                consts = self.constant_propagation(&label_map, i);
            }
            match &ics[i] {
                Operation::Copy(dist, _)
                | Operation::Add(dist, _, _)
                | Operation::Sub(dist, _, _)
                | Operation::Mul(dist, _, _)
                | Operation::Div(dist, _, _) => {
                    if is_const(&mut consts, dist) {
                        let val_string = get_const(&mut consts, dist).to_string();
                        ics[i] = Operation::Copy(dist.clone(), Token::new(val_string));
                    }
                }
                Operation::Print(val) => {
                    if is_const(&mut consts, val) {
                        let val_string = get_const(&mut consts, val).to_string();
                        ics[i] = Operation::Print(Token::new(val_string));
                    }
                }
                Operation::Goto(_) | Operation::IfGoto(..) => {
                    // do constant propagation again
                    if i + 1 < self.internal_code.len() {
                        consts = self.constant_propagation(&label_map, i + 1);
                    }
                }
                _ => (),
            }
        }
        self.internal_code = ics;
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
        assert_eq!(
            tok_strs,
            vec![
                "v200", "=", "200", ";", "if", "(", "v200", "/", "4", "==", "900", ")", "goto",
                "end", ";", ".", ".", "."
            ]
        );
    }

    #[test]
    fn test_add() {
        let src = String::from("result = 100 + 200 - 50;");
        let mut var = VariableMap::new();
        run(src, &mut var);
        let result = var.get(&Token::new(String::from("result")));
        assert_eq!(result, 250);
    }

    #[test]
    fn test_expr() {
        let src = String::from("a = 10; result = tmp = a * 2; ");
        let mut var = VariableMap::new();
        run(src, &mut var);
        let result = var.get(&Token::new(String::from("result")));
        assert_eq!(result, 20);
    }

    #[test]
    fn test_int_var() {
        let src = String::from("result = 1; result = result + result * 2; result = result + 4;");
        let mut var = VariableMap::new();
        run(src, &mut var);
        let result = var.get(&Token::new(String::from("result")));
        assert_eq!(result, 7);
    }

    #[test]
    fn test_goto() {
        let src = String::from("result = 1; goto A; B: result = result + 4; goto C; A: result = result + 2; goto B; C:");
        let mut var = VariableMap::new();
        run(src, &mut var);
        let result = var.get(&Token::new(String::from("result")));
        assert_eq!(result, 7);
    }

    #[test]
    fn test_if() {
        let src = String::from(
            "a = 2; if (a <= 2) { if (a == 1) {} else { result = 10; } } else { a = 0; }",
        );
        let mut var = VariableMap::new();
        run(src, &mut var);
        let result = var.get(&Token::new(String::from("result")));
        assert_eq!(result, 10);
    }

    #[test]
    fn test_for() {
        let src = String::from("sum = 0; i = 0; for (;i <= 10; i = i + 1) { sum = sum + i; }");
        let mut var = VariableMap::new();
        run(src, &mut var);
        let sum = var.get(&Token::new(String::from("sum")));
        assert_eq!(sum, 55);
    }

    #[test]
    fn test_array() {
        let src = String::from("let a[3]; a[1] = 1; a[2] = 2;");
        let mut var = VariableMap::new();
        run(src, &mut var);
        let a = var.array_map.remove(&String::from("a")).unwrap();
        assert_eq!(a, vec![0, 1, 2]);
    }
}
