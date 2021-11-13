use crate::lexer;
use crate::lexer::{Lexer, Token, TokenType};
use crate::var_map::VariableMap;
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub enum Operation {
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
    Println(Token),
    Time,
    Goto(Token),
    IfGoto(Token, Token),          // cond, label
    ArrayNew(Token, Token),        // name, size
    ArraySet(Token, Token, Token), // name, index, val
    ArrayGet(Token, Token, Token), // dist, name, index
}

fn dump_operation(op: &Operation) {
    match op {
        Operation::Copy(ref dist, ref operand) => {
            println!(
                "copy {}, {}",
                lexer::dump_token(dist),
                lexer::dump_token(operand)
            );
        }
        Operation::Add(ref dist, ref lhs, ref rhs) => {
            println!(
                "add {}, {}, {}",
                lexer::dump_token(dist),
                lexer::dump_token(lhs),
                lexer::dump_token(rhs)
            );
        }
        Operation::Sub(ref dist, ref lhs, ref rhs) => {
            println!(
                "sub {}, {}, {}",
                lexer::dump_token(dist),
                lexer::dump_token(lhs),
                lexer::dump_token(rhs)
            );
        }
        Operation::Mul(ref dist, ref lhs, ref rhs) => {
            println!(
                "mul {}, {}, {}",
                lexer::dump_token(dist),
                lexer::dump_token(lhs),
                lexer::dump_token(rhs)
            );
        }
        Operation::Div(ref dist, ref lhs, ref rhs) => {
            println!(
                "div {}, {}, {}",
                lexer::dump_token(dist),
                lexer::dump_token(lhs),
                lexer::dump_token(rhs)
            );
        }
        Operation::Eq(ref dist, ref lhs, ref rhs) => {
            println!(
                "eq {}, {}, {}",
                lexer::dump_token(dist),
                lexer::dump_token(lhs),
                lexer::dump_token(rhs)
            );
        }
        Operation::Ne(ref dist, ref lhs, ref rhs) => {
            println!(
                "ne {}, {}, {}",
                lexer::dump_token(dist),
                lexer::dump_token(lhs),
                lexer::dump_token(rhs)
            );
        }
        Operation::Lt(ref dist, ref lhs, ref rhs) => {
            println!(
                "lt {}, {}, {}",
                lexer::dump_token(dist),
                lexer::dump_token(lhs),
                lexer::dump_token(rhs)
            );
        }
        Operation::Le(ref dist, ref lhs, ref rhs) => {
            println!(
                "le {}, {}, {}",
                lexer::dump_token(dist),
                lexer::dump_token(lhs),
                lexer::dump_token(rhs)
            );
        }
        Operation::Print(ref var) => {
            println!("print {}", lexer::dump_token(var));
        }
        Operation::Println(ref var) => {
            println!("println {}", lexer::dump_token(var));
        }
        Operation::Goto(ref label) => {
            println!("goto {}", lexer::dump_token(label));
        }
        Operation::IfGoto(ref cond, ref label) => {
            println!(
                "ifGoto {}, {}",
                lexer::dump_token(cond),
                lexer::dump_token(label)
            );
        }
        Operation::Time => {
            println!("time");
        }
        Operation::ArrayNew(ref ident, ref size_tok) => {
            println!(
                "arrayNew {}, {}",
                lexer::dump_token(ident),
                lexer::dump_token(size_tok)
            );
        }
        Operation::ArrayGet(ref dist, ref ident, ref index_tok) => {
            println!(
                "arrayGetElem {}, {}, {}",
                lexer::dump_token(dist),
                lexer::dump_token(ident),
                lexer::dump_token(index_tok)
            );
        }
        Operation::ArraySet(ref ident, ref index_tok, ref val_tok) => {
            println!(
                "arraySet {}, {}, {}",
                lexer::dump_token(ident),
                lexer::dump_token(index_tok),
                lexer::dump_token(val_tok)
            );
        }
    }
}

#[derive(PartialEq, Eq)]
enum Block {
    IfElse(Token, Option<Token>),           // L0, L1
    For(Token, Token, Token, usize, usize), // L0, L1, L2, e1_start, e2_start
}

// TODO: this is dirty. Is it better to make an expression parser?
pub struct Parser {
    pos: usize,
    pub lexer: Lexer,
    pub internal_code: Vec<Operation>,
    // temporary strage of operation parameters
    // see the difinition of phrase_compare
    cur_token_param: [Option<Token>; 4],
    cur_expr_param_start_pos: [usize; 4],

    // these two are used only to parse expressions
    expr_pos: usize,
    // the number of variables which is used
    // to store temporay results of calculation
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
    pub fn new(s: String) -> Self {
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
        let ret = Token::new(
            String::from(format!("_tmp{}", self.temp_var_cnt)),
            crate::lexer::TokenType::Ident,
        );
        self.temp_var_cnt += 1;
        ret
    }
    fn make_temp_label(&mut self) -> Token {
        let ret = Token::new(
            String::from(format!("_tmpLabel{}", self.temp_label_cnt)),
            TokenType::Ident,
        );
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
            let op = Operation::Sub(
                tmp.clone(),
                Token::new(String::from("0"), TokenType::NumLiteral),
                self.primary(),
            );
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
        // Easily analyze programs by using each temporary variable once
        // Do not modify below:
        // self.temp_var_cnt = 0;
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

    pub fn compile(&mut self, var: &mut VariableMap) {
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
                self.push_internal_code(Operation::Print(expr0));
            }
            // println
            else if self.phrase_compare(["println", "*e0", ";"]) {
                // TODO: Is it valid that e0 is a string literal?
                // It works because the length of *e0 is at least one.
                let expr0 = self.get_expr_param(0);
                self.push_internal_code(Operation::Println(expr0));
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
                    Token::new(String::from("0"), TokenType::NumLiteral),
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
                        Token::new(String::from("0"), TokenType::NumLiteral),
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

    pub fn dump_internal_code(&self, var_map: &mut VariableMap) {
        let mut label_map: Vec<HashSet<Token>> = vec![HashSet::new(); self.internal_code.len() + 1];

        // show the all labels
        for s in var_map.map.keys() {
            let line = var_map.map.get(s).unwrap();
            label_map[*line as usize].insert(Token::new(s.clone(), TokenType::Ident));
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
                print!("\t");
                dump_operation(&self.internal_code[i])
            }
        }
        println!("-----------------------------------------------------");
    }
}
