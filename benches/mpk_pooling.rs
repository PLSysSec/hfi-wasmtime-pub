use anyhow::Result;
use colorguard::*;
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

fn benchmark_name<'a>(strategy: &InstanceAllocationStrategy) -> &'static str {
    match strategy {
        InstanceAllocationStrategy::OnDemand => "default",
        InstanceAllocationStrategy::Pooling { .. } => "pooling",
    }
}

fn bench_mpk_pooling(
    c: &mut Criterion,
    path: &Path,
    instance_slot_count: usize,
    mpk: bool,
    slot_size: u64,
) {
    let mut group = c.benchmark_group(format!(
        "mpk_pooling: {} instances, mpk: {}, slot_size: {} MiB",
        instance_slot_count,
        mpk,
        slot_size >> 20
    ));

    let strategy = InstanceAllocationStrategy::Pooling {
        strategy: PoolingAllocationStrategy::NextAvailable,
        instance_limits: InstanceLimits {
            memory_pages: 2,
            count: instance_slot_count as u32,
            ..Default::default()
        },
    };

    let state = {
        let mut config = Config::default();
        config.allocation_strategy(strategy.clone());
        // disable guard pages
        config.static_memory_guard_size(0);
        config.guard_before_linear_memory(false);
        config.static_memory_maximum_size(slot_size);
        config.static_memory_forced(true);
        config.mpk_pooling(mpk);

        let pkeys = if mpk {
            colorguard_allocate_pkeys()
        } else {
            [0; 16]
        };

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
            let mut instances =
                std::iter::repeat_with(move || (engine_clone.clone(), pre_clone.clone()))
                    .take(instance_slot_count as usize)
                    .collect::<Vec<_>>()
                    .iter() // TODO: par_iter
                    .enumerate()
                    .map(|(idx, (engine, pre))| {
                        let mut store = store(engine);
                        let instance = pre.instantiate(&mut store).unwrap();
                        let mem = instance.get_memory(&mut store, "memory").unwrap();

                        let linmem_base = mem.data_ptr(&store);
                        let pkey = ((idx % 15) + 1) as u32; // 1-15
                        let f = instance.get_func(&mut store, "_start").unwrap();
                        (store, instance, f, pkey)
                    })
                    .collect::<Vec<_>>();

            for (ref mut store, instance, f, pkey) in instances.iter_mut() {
                colorguard_enter_sbox(*pkey);
                let s: &mut wasmtime::Store<wasmtime_wasi::WasiCtx> = store;
                f.call(s, &[], &mut []).unwrap();
            }
        });
    });

    if mpk {
        colorguard_deallocate_pkeys(pkeys);
    }

    group.finish();
}

fn bench_pooling(c: &mut Criterion) {
    let path = Path::new("benches/instantiation/toobig.wat");
    let instance_slot_count = 8;
    let mpk = true;
    let slot_size = 1 << 30;
    bench_mpk_pooling(c, &path, instance_slot_count, mpk, slot_size);
}

criterion_group!(benches, bench_pooling);
criterion_main!(benches);
