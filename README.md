# haribote-lang
[![CircleCI](https://circleci.com/gh/tamaroning/haribote-lang/tree/main.svg?style=shield)](https://circleci.com/gh/tamaroning/haribote-lang/tree/main)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

haribote-lang is a simple and fast programming language for education originally made by Mr.Kawai.  
This repository is a remodelled implementation in Rust of the original version, [Creating programming languages in 10 days](http://essen.osask.jp/?a21_txt01).  

## Features
- This repository contains hrb, haribote-lang interpreter
- hrb runs in two modes, normal mode and interactive mode (a.k.a. REPL)
- Input source code is converted into internal code
- and is optimized it in several ways

For further information, See [Optimization Strategy](#Optimization-Strategy) or [My Blog(ja)](https://tamaron.hatenablog.com/entry/2021/11/20/165929).

# Build
haribote-lang run on Windows, OSX, Linux, UNIX.  

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

# Usage
- `hrb [OPTIONS] FILEPATH`: Run the program
- `hrb [OPTIONS]`: Run in interactive mode
- `hrb help`: show the usage

### Options
- `-emit-ir`: Display the intermidiate representation
- `-no-optimize`: Doesn't optimize the program
- `-no-exec`: Doesn't execute the program

# Demo

In interactive mode, you can type an expression to check the result
```
?> hrb
haribote-lang version 1.1.1
Running in Interactive mode
Type "run <filepath>" to load and run the file.
>>> print "Hello World\n";
Hello World
>>> a = 15; b = 20;
>>> a * b
300
>>> exit
?>
```


```
?> hrb ./example/fibo.hrb`
Fibo_0 = 1
Fibo_1 = 1
Fibo_2 = 2
Fibo_3 = 3
Fibo_4 = 5
Fibo_5 = 8
Fibo_6 = 13
Fibo_7 = 21
Fibo_8 = 34
Fibo_9 = 55
Fibo_10 = 89
Fibo_11 = 144
?>
```

```
?> hrb -emit-ir example/calc.hrb
Optimizing...
--------------- Dump of internal code ---------------
        copy a, i32 1
        copy b, i32 3
        copy _tmp0, i32 3
        copy _tmp1, i32 4
        copy c, i32 4
        copy _tmp0, i32 8
        copy _tmp1, i32 10
        copy c, i32 10
-----------------------------------------------------
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
               | "print" (expr | <Str>) ";"
               | "println" (expr | <Str>) ";"

array-decl  ::= "let" <Ident> "[" expr "]" ";"

if-else     ::= if-goto | if-else-sub
if-goto     ::= "if" "(" expr ")" goto-stmt
if-else-sub ::= "if" "(" expr ")" "{" top* "}" ( "else" "{" top* "}" )?

for         ::= "for" "(" expr? ";" expr? ";" expr? ")" "{" top* "}"

goto-stmt   ::= "goto" <Ident> ";"

expr        ::= assign
assign      ::= equality ( ("=" | "+=" | "-=" | "*=" | "/=") expr )?
equality    ::= relational ( ( "==" | "!=" ) relational )*
relational  ::= add        ( (  "<" | "<=" ) add        )*
add         ::= mul        ( (  "+" | "-"  ) mul        )*
mul         ::= unary      ( (  "*" | "/"  ) unary      )*
unary       ::= ("+" | "-")? primary
primary     ::= <Num> | <Ident> ( "[" expr "]" )?

```

# Intermidiate Representation
Intermidiate Representation (IR) is a low-level code of haribote language.  
Optimizations are taken place on IR.  
In order to learn how IR is implemented, see a struct `Operation` in /src/parser.rs. 

# Optimization Strategy

hrb supports the following optimization methods:
- [Constant Folding & Constant Propagation](#Constant-Folding-Constant-Propagation)<!--remove & from link-->
- [Removing Unreachable Operations](#Removing-Unreachable-Operations)
- [Peekhole Optimization](#Peekhole-Optimization)
    - [Jump Chain Optimization](#Jump-Chain-Optimization)

Basic strategy is as follows:

1. Build a control-flow graph
2. Data-flow analysis
3. Change code

## Constant Folding &Constant Propagation
1. Let each CFG node have a constant variable table.
2. Information of constant variables moves to other nodes along the control flow. (One of the good ways is using a queue.)
3. Replace arithmetic operations with copy operations by using information of the final stete.

## Removing Unreachable Operations
1. Let each CFG node n have a boolean value b[n].
2. Set all b[n]s falses.
3. Set b[entry point] true.
4. Do BFS and set every b[reachable node] true.
5. Remove n such that b[n] = false.

## Peekhole Optimization
It's unnecessary to build a CFG.

### Jump Chain Optimization
1. Search a `goto(jump) L` operation. L is a label.
2. If the distination label of L starts with a goto(jump) operation, get the destination of L.
3. Repeat step 2. Finally get the final destination of label D. 
4. If detects a cyclic control-flow in step 3, do nothing and finish.
5. Replace the original `goto(jump) L` with `goto(jump) D`. 

<!--
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
-->

# References
- http://essen.osask.jp/?a21_txt01
- http://pages.cs.wisc.edu/~horwitz/CS704-NOTES/2.DATAFLOW.html
