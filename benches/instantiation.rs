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

fn instantiate(pre: &InstancePre<WasiCtx>, engine: &Engine, slot: usize) -> Result<()> {
    let mut store = store(engine);
    let instance = pre.instantiate(&mut store, slot)?;
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

fn bench_deferred_cleanup(
    c: &mut Criterion,
    path: &Path,
    instance_slot_count: usize,
    batching: bool,
    slot_size: u64,
    guard: bool,
) {
    let mut group = c.benchmark_group(format!(
        "deferred_cleanup: {} instances, batching: {}, slot_size: {} MiB",
        instance_slot_count,
        batching,
        slot_size >> 20
    ));

    let strategy = InstanceAllocationStrategy::Pooling {
        strategy: Default::default(),
        instance_limits: InstanceLimits {
            memory_pages: 2,
            count: instance_slot_count as u32,
            ..Default::default()
        },
    };

    let state = {
        let mut config = Config::default();
        config.allocation_strategy(strategy.clone());
        if guard {
            config.static_memory_guard_size(1 << 31);
            config.guard_before_linear_memory(true);
        } else {
            config.static_memory_guard_size(0);
            config.guard_before_linear_memory(false);
        }
        config.static_memory_maximum_size(slot_size);
        config.static_memory_forced(true);
        config.deferred_dealloc(batching);

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

    let id = BenchmarkId::new(benchmark_name(&strategy), format!("{}", path.display()));

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
                .enumerate()
                .collect::<Vec<_>>()
                .par_iter()
                .for_each(|(i, (engine, pre))| {
                    instantiate(&pre, &engine, *i).expect("failed to instantiate module");
                });

            // In batching case, this does one big madvise(); in
            // non-batching case, it just returns slots to the free
            // pool. (We want to return slots to the free pool in the
            // same way in both cases to control for different
            // allocation patterns in the two cases.)
            engine.deferred_cleanup();
        });
    });

    group.finish();
}

fn bench_instantiation(c: &mut Criterion) {
    for file in std::fs::read_dir("benches/instantiation").unwrap() {
        let path = file.unwrap().path();
        for &(batching, instance_slot_count, slot_size, guard) in &[
            //            (false, 1000, 1 << 32, true),
            //            (true, 1000, 1 << 17, false),
            (false, 2000, 1 << 32, true),
            (true, 2000, 1 << 20, false),
            /*            (false, 3000, 1 << 32, true),
            (true, 3000, 1 << 17, false),
            (false, 4000, 1 << 32, true),
            (true, 4000, 1 << 17, false),
            (false, 5000, 1 << 32, true),
            (true, 5000, 1 << 17, false),
            (false, 10000, 1 << 32, true),
            (true, 10000, 1 << 17, false),
            (false, 100000, 1 << 32, true),
            (true, 100000, 1 << 17, false),*/
        ] {
            bench_deferred_cleanup(c, &path, instance_slot_count, batching, slot_size, guard);
        }
    }
}

criterion_group!(benches, bench_instantiation);
criterion_main!(benches);
