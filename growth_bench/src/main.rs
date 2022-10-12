use std::path::Path;
use wasmtime::*;

fn get_engine_and_module(path: &Path) -> (Engine, Module) {
    let mut config = Config::default();
    config.allocation_strategy(InstanceAllocationStrategy::Pooling {
        strategy: Default::default(),
        instance_limits: InstanceLimits {
            memory_pages: 2048, // 128 MiB
            ..Default::default()
        },
    });

    let engine = Engine::new(&config).unwrap();
    let module = Module::from_file(&engine, path).unwrap();
    (engine, module)
}

fn instantiate_and_run(engine: &Engine, module: &Module) {
    let mut store = Store::new(engine, ());
    let instance = Instance::new(&mut store, module, &[]).unwrap();
    let func = instance.get_func(&mut store, "_start").unwrap();
    func.call(&mut store, &[], &mut []).unwrap();
}

fn main() {
    let path = Path::new("./benchmark.wasm");
    let (engine, module) = get_engine_and_module(&path);

    for i in 0..1_000 {
        if i % 100 == 0 {
            println!("{}", i);
        }
        instantiate_and_run(&engine, &module);
    }
}
