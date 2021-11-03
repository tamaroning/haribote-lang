# haribote-lang
haribote-lang is a simple and fast programming language originally made by Mr.Kawai.  
This repository is derived from the original version.  

## Features
- This repository contains hrb, haribote-lang interpreter
- hrb runs in two modes, normal mode and interactive mode (a.k.a. REPL)
- Input source code is converted into internal code
- and optimized in the following ways :
    - Constant Folding & Constant Propagation
    - Jump Chain Optimization

# Build
haribote-lang run on Windows, OSX, and Linux.  

```sh
git clone https://github.com/tamaroning/haribote-lang.git
cd haribote-lang
cargo build --release
```

# Run
Run a .hrb file:
``` sh
./target/release/hrb <filepath>
```

Run in an interactive mode:
``` sh
./target/release/hrb
```

# Demo
```
?> ./hrb
haribote-lang interactive mode
> prints "Hello World\n";  
Hello World
> a = 15; b = 20;       
> c = a * b;  
> prints "answer is "; print c;
answer is 300
> exit;
?>
```

```
?> hrb ./example/fibo.hrb`
Fibo = 1
Fibo = 1
Fibo = 2
Fibo = 3
Fibo = 5
Fibo = 8
Fibo = 13
Fibo = 21
Fibo = 34
Fibo = 55
Fibo = 89
Fibo = 144
?>
```

# Grammar

## Lexical elements

- Num : Decimal numbers (0, 100, 53, 1024)
- Ident : Identifiers which starts with alphabet (abc, ABc123)
- Str : Strings encloses in double quotes ("Hello Hari-bote ", "World\n")

## Definition by EBNF
```
program     ::= top*

top         ::= label | stmt

label       ::= <Ident> ":"

stmt        :: = array-decl
               | if-else
               | for
               | goto-stmt
               | expr? ";"
               | "print" expr ";"
               | "prints" <Str> ";"

array-decl  ::= "let" <Ident> "[" expr "]" ";"

if-else     ::= if-goto | if-else-sub
if-goto     ::= "if" "(" expr ")" goto-stmt
if-else-sub ::= "if" "(" expr ")" "{" top* "}" ( "else" "{" top* "}" )?

for         ::= "for" "(" expr? ";" expr? ";" expr? ")" "{" top* "}"

goto-stmt   ::= "goto" <Ident> ";"

expr        ::= assign
assign      ::= equality ( "=" expr )?
equality    ::= relational ( ( "==" | "!=" ) relational )*
relational  ::= add        ( (  "<" | "<=" ) add        )*
add         ::= mul        ( (  "+" | "-"  ) mul        )*
mul         ::= unary      ( (  "*" | "/"  ) unary      )*
unary       ::= ("+" | "-")? primary
primary     ::= <Num> | <Ident> ( "[" expr "]" )?

```

# Commit Logs
You can see the commit log to follow the steps of implementation.  
The steps from 1 to 8 are the same as those of [the original version](http://essen.osask.jp/?a21_txt01).  

| Step | features |
| ---- | ---- |
| 1 | A very simple language. |
| 2 | Multi-character variable name. Skips spaces. |
| 3 | Conditional branch. Run loops. |
| 4 | REPL (interactive mode). |
| 5 | Speed up the program. |
| 6 | Speed up the program. |
| 7 | Expression. |
| 8 | if-else & for statement. Optimize goto. |
| 9 | Array (declaration, assignment, random access). |
| 10 | Constant Folding & Constant Propagation. |

# References
- http://essen.osask.jp/?a21_txt01
