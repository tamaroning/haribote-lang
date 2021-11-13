#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Token {
    pub string: String,
    pub ty: TokenType,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TokenType {
    Simbol,
    Ident,
    NumLiteral,
    StrLiteral,
}

impl Token {
    pub fn new(s: String, ty: TokenType) -> Self {
        Token {
            string: s,
            ty: ty,
        }
    }

    pub fn matches(&self, s: &str) -> bool {
        self.string == s
    }
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
pub struct Lexer {
    txt: String,
    pos: usize,
    pub tokens: Vec<Token>,
}

impl Lexer {
    pub fn new(prg: String) -> Self {
        Lexer {
            txt: prg,
            pos: 0,
            tokens: Vec::new(),
        }
    }

    fn next_char(&self) -> char {
        self.txt[self.pos..].chars().next().unwrap()
    }

    pub fn lex(&mut self) {
        while self.pos < self.txt.len() {
            let start_pos = self.pos;

            // skip whitespace
            if is_whitespace(self.next_char()) {
                self.pos += 1;
                continue;
            }

            // string literals
            if self.next_char() == '"' {
                self.pos += 1;

                // whether double quotation is found
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
                self.tokens.push(Token::new(s, TokenType::StrLiteral));
                continue;
            }

            let tok_ty;
            if is_one_char_symbol(self.next_char()) {
                self.pos += 1;
                tok_ty = TokenType::Simbol
            } else if self.next_char().is_numeric() {
                // TODO: check if the token is a numerical litaral
                self.pos += 1;
                while self.pos < self.txt.len() && self.next_char().is_numeric() {
                    self.pos += 1;
                }
                tok_ty = TokenType::NumLiteral;
            } 
            else if self.next_char().is_alphabetic() {
                self.pos += 1;
                while self.pos < self.txt.len() && self.next_char().is_alphanumeric() {
                    self.pos += 1;
                }
                tok_ty = TokenType::Ident;
            } else if is_normal_symbol(self.next_char()) {
                self.pos += 1;
                while self.pos < self.txt.len() && is_normal_symbol(self.next_char()) {
                    self.pos += 1;
                }
                tok_ty = TokenType::Simbol
            } else {
                println!("Syntax error : '{}'", self.next_char());
                std::process::exit(0);
            }
            let s = self.txt[start_pos..self.pos].to_string();
            self.tokens.push(Token::new(s, tok_ty));
        }
        self.tokens.push(Token::new(String::from("."), TokenType::Simbol));
        self.tokens.push(Token::new(String::from("."), TokenType::StrLiteral));
        self.tokens.push(Token::new(String::from("."), TokenType::StrLiteral));
    }
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
            "v200", "=", "200", ";", "if", "(", "v200", "/", "4", "==", "900", ")", "goto", "end",
            ";", ".", ".", "."
        ]
    );
}
