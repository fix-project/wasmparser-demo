(module
  (func $add (param i32 i32) (result i32)
    local.get 0
    local.get 1
    i32.add)

  (func $square (param f64) (result f64)
    local.get 0
    local.get 0
    f64.mul))
