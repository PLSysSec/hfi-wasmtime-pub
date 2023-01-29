(module
  (memory (export "memory") 2 15625)  ;; 1GB
  (data 0 (i32.const 0) "asdf")
  (data 0 (i32.const 65536) "jkl;")
  (func (export "_start")
        i32.const 0
        i32.const 0
        i32.store
        i32.const  1073742824 
        i32.const 0
        i32.store))
