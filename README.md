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
😁

```
❯ time cargo run example/fib.lisp --release
    Finished release [optimized] target(s) in 0.0 secs
     Running `target/release/secd example/fib.lisp`
832040
832040
cargo run example/fib.lisp --release  3.11s user 0.04s system 99% cpu 3.176 total
```

