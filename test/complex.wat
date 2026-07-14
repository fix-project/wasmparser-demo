(module
  ;; imported function - also gets a type index, from the import section
  (import "env" "log" (func $log (param i32)))

  ;; two functions sharing the exact same signature -> same type index
  (func $add (param i32 i32) (result i32)
    local.get 0
    local.get 1
    i32.add)

  (func $sub (param i32 i32) (result i32)
    local.get 0
    local.get 1
    i32.sub)

  ;; different signature -> new type index
  (func $square (param f64) (result f64)
    local.get 0
    local.get 0
    f64.mul)

  ;; no params or results at all -> another distinct type
  (func $noop)

  ;; calls another function defined in this module
  (func $add_and_log (param i32 i32)
    local.get 0
    local.get 1
    call $add
    call $log)

  (export "add" (func $add))
  (export "square" (func $square)))
