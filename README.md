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
😓

```
❯ time cargo run ../example/fib.lisp --release
    Finished release [optimized] target(s) in 0.0 secs
     Running `/Users/tokiya/Documents/rust/secd/target/release/secd ../example/fib.lisp`
832040
cargo run ../example/fib.lisp --release  12.99s user 0.06s system 99% cpu 13.081 total
```

# see also
[fast and better](https://github.com/KeenS/secd-rs/tree/non-cloning)
