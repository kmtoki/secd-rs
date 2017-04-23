(let                    
  z
    (lambda f
      ((lambda x
        (f
          (lambda y ((x x) y))))
       (lambda x
        (f
          (lambda y ((x x) y))))))

  (let 
    sum
      (lambda f
        (lambda x
          (if (eq x 1)
            x
            (+ x (f (- x 1))))))
    
    ((z sum) 10)))
