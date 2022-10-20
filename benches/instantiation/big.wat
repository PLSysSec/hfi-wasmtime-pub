(module
  (memory 2 2)  ;; 128 KiB
  (func (export "_start")
        i32.const 0
        i32.const 0
        i32.store
        i32.const 65536
        i32.const 0
        i32.store))
