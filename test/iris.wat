(module
  (import "env" "log" (func $log (param i32)))

  (func $add (param i32 i32) (result i32)
    (local i32 i64 i32)
    local.get 0
    local.get 1
    i32.add)

  (func $sub (param i32 i32) (result i32)
    local.get 0
    local.get 1
    i32.sub)

  (func $square (param f64) (result f64)
    local.get 0
    local.get 0
    f64.mul)

  (func $noop)

  ;; locals declared, distinct from params
  (func $sum_of_squares (param i32 i32) (result i32)
    (local $a_squared i32)
    (local $b_squared i32)
    (local $total i32)

    local.get 0
    local.get 0
    i32.mul
    local.set $a_squared

    local.get 1
    local.get 1
    i32.mul
    local.set $b_squared

    local.get $a_squared
    local.get $b_squared
    i32.add
    local.set $total

    local.get $total)

  ;; multiple locals of the same type declared together, plus a mixed-type local
  (func $mixed_locals (param i32) (result f64)
    (local i32 i32 i32)  ;; three i32 locals, indices 1, 2, 3
    (local f64)          ;; one f64 local, index 4

    local.get 0
    i32.const 1
    i32.add
    local.set 1

    local.get 1
    f64.convert_i32_s
    local.set 4

    local.get 4)

  (func $add_and_log (param i32 i32)
    local.get 0
    local.get 1
    call $add
    call $log)

  (export "add" (func $add))
  (export "sum_of_squares" (func $sum_of_squares)))
