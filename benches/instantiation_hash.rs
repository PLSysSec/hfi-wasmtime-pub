use std::path::PathBuf;
use anyhow::Result;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use rayon::prelude::*;
use std::path::Path;
use std::sync::Arc;
use wasmtime::*;
use wasmtime_wasi::{sync::WasiCtxBuilder, WasiCtx};

mod instantiation_utils;
use instantiation_utils::*;

//hash=2CF24DBA5FB0A30E26E83B2AC5B9E29E1B161E5C1FA7425E73043362938B9824
fn store(engine: &Engine) -> Store<WasiCtx> {
    let msg = include_str!("data/test_msg.txt");
    let wasi = WasiCtxBuilder::new().arg("hash").expect("couldn't set program name in argv").arg(msg).expect("couldn't load msg into argv?").arg("2CF24DBA5FB0A30E26E83B2AC5B9E29E1B161E5C1FA7425E73043362938B9824").expect("Couldn't set argv?").build();
    //wasi.push_arg("20");
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

fn bench_hash(
    c: &mut Criterion,
    path: &Path,
    instance_slot_count: usize,
    batching: bool,
    slot_size: u64,
    guard: bool,
) {
    let mut group = c.benchmark_group(format!(
        "hash: {} instances, batching: {}, slot_size: {} MiB",
        instance_slot_count,
        batching,
        slot_size >> 20
    ));

    // HFI: number of instances in the address space.
    let strategy = InstanceAllocationStrategy::Pooling {
        strategy: Default::default(),
        instance_limits: InstanceLimits {
            memory_pages: 16,
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
        linker.func_wrap("env", "server_module_string_result", 
            |caller: Caller<'_, WasiCtx>, string_resp: u32, bytes: u32| {
                server_module_string_result(caller, string_resp, bytes)
            }
        ).unwrap();
        linker.func_wrap("env", "server_module_bytearr_result", 
            |caller: Caller<'_, WasiCtx>, string_resp: u32, bytes: u32| {
                server_module_bytearr_result(caller, string_resp, bytes)
            }
        ).unwrap();
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
                .collect::<Vec<_>>()
                .par_iter()
                .for_each(|(engine, pre)| {
                    instantiate(&pre, &engine).expect("failed to instantiate module");
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

fn bench_instantiation_hash(c: &mut Criterion) {
    // for file in std::fs::read_dir("benches/instantiation").unwrap() {
        // let path = file.unwrap().path();
        let mut path = PathBuf::new();
        path.push("benches/modules/msghash_check_c.wasm");
        // let path = std::fs::canonicalize(path).unwrap();
        for &(batching, instance_slot_count, slot_size, guard) in &[
            (false, 1, 1 << 32, true),
            // (true, 1000, 1 << 20, false),
            // (false, 2000, 1 << 32, true),
            // (true, 2000, 1 << 20, false),
            // (false, 3000, 1 << 32, true),
            // (true, 3000, 1 << 20, false),
            // (false, 4000, 1 << 32, true),
            // (true, 4000, 1 << 20, false),
            // (false, 5000, 1 << 32, true),
            // (true, 5000, 1 << 20, false),
            // (false, 10000, 1 << 32, true),
            // (true, 10000, 1 << 20, false),
            // (false, 100000, 1 << 32, true),
            // (true, 100000, 1 << 20, false),
        ] {
            bench_hash(c, &path, instance_slot_count, batching, slot_size, guard);
        }
    // }
}

criterion_group!(benches_hash, bench_instantiation_hash);
criterion_main!(benches_hash);
