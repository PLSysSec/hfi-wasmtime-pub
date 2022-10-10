(module
  (memory 100 100)
  (data 0 (i32.const 0) "asdf")
  (data 0 (i32.const 1048576) "jkl;")
  (func (export "_start")
        i32.const 0
        i32.const 0
        i32.store
        i32.const 4096
        i32.const 0
        i32.store
        i32.const 8192
        i32.const 0
        i32.store
        i32.const 12288
        i32.const 0
        i32.store
        i32.const 16384
        i32.const 0
        i32.store
        i32.const 65536
        i32.const 0
        i32.store
        i32.const 131072
        i32.const 0
        i32.store
        i32.const 262144
        i32.const 0
        i32.store
        i32.const 1048576
        i32.const 0
        i32.store)
