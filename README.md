# SECD Machine
pure lisp interpreter by Rust and SECD Machine

## usage
```
cargo run <file>
```

## spec
```lisp
(let <id> <expr> <body>)
(letrec <id> <expr> <body>)
(lambda <<id> | (<id>+)> <body>)
(if <bool> <then> <else>)
(eq <expr> <expr>)
(cons <expr> <expr>)
(car <cons>)
(cdr <cons>)
(+ <int> <int>)
(- <int> <int>)
(puts <expr>)
```

## time
😱

```
cargo run example/fib.lisp  191.92s user 0.46s system 99% cpu 3:13.34 total
```
