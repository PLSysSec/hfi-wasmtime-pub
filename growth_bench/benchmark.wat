(module
  (memory (export "memory") 0 2048)
  (func (export "_start")
        (local $i i32)
        (local.set $i (i32.const 1))
        (loop $l
              (drop (memory.grow (i32.const 1)))
              (i32.store (i32.sub (i32.mul (local.get $i) (i32.const 65536))
                                  (i32.const 4))
                         (i32.const 1234))
              (local.set $i (i32.add (local.get $i) (i32.const 1)))
              (br_if $l (i32.le_u (local.get $i) (i32.const 2048))))))
