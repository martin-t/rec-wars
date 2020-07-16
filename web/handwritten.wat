(module
  (func $myadd (param $a i32) (param $b i32) (result i32)
    ;;(local.get $a) (local.get $b) (i32.add))
    local.get $b
    (i32.add (local.get $a)))
  (export "exp_add" (func $myadd)))
