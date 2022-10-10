use anyhow::Result;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use rayon::prelude::*;
use std::path::Path;
use std::sync::Arc;
use wasmtime::*;
use wasmtime_wasi::{sync::WasiCtxBuilder, WasiCtx};

fn store(engine: &Engine) -> Store<WasiCtx> {
    let wasi = WasiCtxBuilder::new().build();
    Store::new(engine, wasi)
}

fn instantiate(pre: &InstancePre<WasiCtx>, engine: &Engine) -> Result<()> {
    let mut store = store(engine);
    let instance = pre.instantiate(&mut store)?;
    instance
        .get_func(&mut store, "_start")
        .unwrap()
        .call(&mut store, &[], &mut [])?;
    Ok(())
}

fn benchmark_name<'a>(strategy: &InstanceAllocationStrategy) -> &'static str {
    match strategy {
        InstanceAllocationStrategy::OnDemand => "default",
        InstanceAllocationStrategy::Pooling { .. } => "pooling",
    }
}

fn bench_deferred_cleanup(c: &mut Criterion, path: &Path) {
    let mut group = c.benchmark_group("deferred_cleanup");

    // Data points to try:
    // - instance_slot_count = 1000, static_memory_maximum_size = 1 << 32 (4 GiB)
    // - instance_slot_count = 10000, static_memory_maximum_size = 1 << 32 (4 GiB)
    // - instance_slot_count = 10000, static_memory_maximum_size = 1 << 21 (2 MiB)
    // - instance_slot_count = 100000, static_memory_maximum_size = 1 << 21 (2 MiB)
    // - instance_slot_count = 1000000, static_memory_maximum_size = 1 << 21 (2 MiB)

    // HFI: number of instances in the address space.
    let instance_slot_count = 10000;
    let strategy = InstanceAllocationStrategy::Pooling {
        strategy: Default::default(),
        instance_limits: InstanceLimits {
            memory_pages: 32,
            count: instance_slot_count,
            ..Default::default()
        },
    };

    let state = {
        let mut config = Config::default();
        config.allocation_strategy(strategy.clone());
        // 8 GiB per instance: 4 GiB heap, 2 GiB guards on either side
        // (production config).
        // HFI: change to 1 << 16 guard, 1 << 21 max size to allocate smaller.
        config.static_memory_guard_size(1 << 16);
        config.guard_before_linear_memory(true);
        config.static_memory_maximum_size(1 << 21);
        config.static_memory_forced(true);

        let engine = Engine::new(&config).expect("failed to create engine");
        let module = Module::from_file(&engine, path).expect("failed to load WASI example module");
        let mut linker = Linker::new(&engine);
        wasmtime_wasi::add_to_linker(&mut linker, |cx| cx).unwrap();
        let pre = Arc::new(
            linker
                .instantiate_pre(&mut store(&engine), &module)
                .expect("failed to pre-instantiate"),
        );
        (engine, pre)
    };

    let id = BenchmarkId::new(
        benchmark_name(&strategy),
        format!("deferred_cleanup on {}", path.display()),
    );

    group.bench_function(id, move |b| {
        let (ref engine, ref pre) = state;

        // Now that our background work is configured we can
        // benchmark the amount of time it takes to instantiate this
        // module.
        b.iter(move || {
            let engine_clone = engine.clone();
            let pre_clone = pre.clone();
            std::iter::repeat_with(move || (engine_clone.clone(), pre_clone.clone()))
                .take(instance_slot_count as usize)
                .collect::<Vec<_>>()
                .par_iter()
                .for_each(|(engine, pre)| {
                    instantiate(&pre, &engine).expect("failed to instantiate module");
                });
            engine.deferred_cleanup();
        });
    });

    group.finish();
}

fn bench_instantiation(c: &mut Criterion) {
    for file in std::fs::read_dir("benches/instantiation").unwrap() {
        let path = file.unwrap().path();
        bench_deferred_cleanup(c, &path);
    }
}

criterion_group!(benches, bench_instantiation);
criterion_main!(benches);
