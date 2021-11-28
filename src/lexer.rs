use crate::error::error_exit;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Token {
    pub string: String,
    pub ty: TokenType,
    pub line: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TokenType {
    Simbol,
    Ident,
    NumLiteral(i32),
    StrLiteral,
}

pub fn dump_token(tok: &Token) -> String {
    match &tok.ty {
        TokenType::Simbol => {
            format!("{}(Simbol)", tok.string)
        }
        TokenType::Ident => tok.string.to_string(),
        TokenType::NumLiteral(_) => {
            format!("i32 {}", tok.string)
        }
        TokenType::StrLiteral => {
            format!("Str \"{}\"", tok.string)
        }
    }
}

impl Token {
    pub fn new(s: String, ty: TokenType) -> Self {
        Token {
            string: s,
            ty,
            line: None,
        }
    }

    pub fn new_with_line_num(s: String, ty: TokenType, line: i32) -> Self {
        Token {
            string: s,
            ty,
            line: Some(line),
        }
    }

    pub fn new_num(n: i32, line: Option<i32>) -> Self {
        Token {
            string: n.to_string(),
            ty: TokenType::NumLiteral(n),
            line,
        }
    }

    pub fn matches(&self, s: &str) -> bool {
        self.string == s
    }
}

fn is_whitespace(c: char) -> bool {
    matches!(c, ' ' | '\n' | '\t' | '\r')
}

fn is_one_char_symbol(c: char) -> bool {
    matches!(c, '(' | ')' | '{' | '}' | '[' | ']' | ';' | ',')
}

fn is_normal_symbol(c: char) -> bool {
    matches!(
        c,
        '=' | '+'
            | '-'
            | '*'
            | '/'
            | '!'
            | '%'
            | '&'
            | '~'
            | '|'
            | '<'
            | '>'
            | '?'
            | ':'
            | '.'
            | '#'
    )
}

#[derive(Debug)]
pub struct Lexer {
    txt: String,
    pos: usize,
    line: i32,
    pub tokens: Vec<Token>,
}

impl Lexer {
    pub fn new(prg: String) -> Self {
        Lexer {
            txt: prg,
            pos: 0,
            line: 0,
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
                if self.next_char() == '\n' {
                    self.line += 1;
                }
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
                    error_exit(String::from("Lexer error: Unmatched '\"'"));
                }
                let mut s = self.txt[start_pos + 1..self.pos - 1].to_string();
                s = s.replace("\\n", "\n");
                self.tokens.push(Token::new_with_line_num(
                    s,
                    TokenType::StrLiteral,
                    self.line,
                ));
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
                let s = self.txt[start_pos..self.pos].to_string();
                tok_ty = TokenType::NumLiteral(s.parse::<i32>().unwrap());
            } else if self.next_char().is_alphabetic() {
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
            self.tokens
                .push(Token::new_with_line_num(s, tok_ty, self.line));
        }
        // In case the input lacks a semicolon at the end, push a semicolon
        self.tokens
            .push(Token::new(String::from(";"), TokenType::Simbol));
        self.tokens
            .push(Token::new(String::from(""), TokenType::Simbol));
        self.tokens
            .push(Token::new(String::from(""), TokenType::StrLiteral));
        self.tokens
            .push(Token::new(String::from(""), TokenType::StrLiteral));
    }
}

#[cfg(test)]
mod lexer_tests {
    use super::*;

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
                "end", ";", ";", "", "", ""
            ]
        );
    }
}
