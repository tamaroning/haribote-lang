# Haribote-lang
haribote-lang is simple and fast programming language.  
This repository is for my learning interpreters.  

# Build
```sh
git clone https://github.com/tamaroning/haribote-lang.git
cd haribote-lang
cargo build --release
```

# Run
To run a .hrb file:
``` sh
./target/release/hrb <filepath>
```

To run with interactive mode:
``` sh
./target/release/hrb
```

# Features
You can see the commit log to follow the steps of implementation.  

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

# Reference
- http://essen.osask.jp/?a21_txt01
