#lang rosette

(define-symbolic* h1 integer?)
(define-symbolic* h2 integer?)

(define (sketch x)
  (* (+ x h1) (+ x h2)))
(define (spec x)
  (+ (* x x) (* 2 x) 1))

(define-symbolic* x integer?)
(solve (assert (forall (list x) (= (sketch x) (spec x)))))
