(letrec fib
  (lambda n
    (if (eq n 0)
      0
    (if (eq n 1)
      1
    (+ (fib (- n 1)) (fib (- n 2))))))
  (puts (fib 30)))
