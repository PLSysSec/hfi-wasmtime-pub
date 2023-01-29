use colorguard::*;
use rayon::prelude::*;
use std::env;
use std::path::Path;
use std::sync::Arc;
use wasmtime::*;
use wasmtime_wasi::{sync::WasiCtxBuilder, WasiCtx};

fn store(engine: &Engine) -> Store<WasiCtx> {
    let wasi = WasiCtxBuilder::new().build();
    Store::new(engine, wasi)
}

fn bench_mpk_pooling(
    path: &Path,
    instance_slot_count: usize,
    mpk: bool,
    slot_size: u64,
    num_engines: usize,
) {
    println!(
        "mpk_pooling: {} instances, mpk: {}, slot_size: {} MiB",
        instance_slot_count,
        mpk,
        slot_size >> 20
    );

    let strategy = InstanceAllocationStrategy::Pooling {
        strategy: PoolingAllocationStrategy::NextAvailable,
        instance_limits: InstanceLimits {
            memory_pages: 2,
            count: instance_slot_count as u32,
            ..Default::default()
        },
    };

    let mut config = Config::default();
    config.allocation_strategy(strategy.clone());
    if mpk {
        config.static_memory_guard_size(0);
        config.guard_before_linear_memory(false);
    } else {
        config.static_memory_guard_size(1 << 31);
        config.guard_before_linear_memory(true);
    }
    config.static_memory_maximum_size(slot_size);
    config.static_memory_forced(true);
    config.mpk_pooling(mpk);

    let pkeys = if mpk {
        colorguard_allocate_pkeys()
    } else {
        [0; 16]
    };

    let mut engines = Vec::new();
    for idx in 0..num_engines {
        println!("count: {:?}", (idx + 1) * instance_slot_count);
        let engine = Engine::new(&config).expect("failed to create engine");
        engines.push(engine);
    }

    if mpk {
        colorguard_deallocate_pkeys(pkeys);
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if !args.len() == 4 {
        println!("Usage: cargo run <path> <num_sandboxes per engine> <num engines> <mpk or no>");
        std::process::exit(1);
    }
    // ../benches/instantiation/toobig.wat
    let p = &args[1];
    // 8
    let instance_slot_count = (&args[2]).parse::<usize>().unwrap();
    let path = Path::new(p);
    let num_engines = (&args[3]).parse::<usize>().unwrap();
    let mpk = &args[4] == "mpk";
    let slot_size = if mpk { 1 << 29 } else { 1 << 32 };
    bench_mpk_pooling(&path, instance_slot_count, mpk, slot_size, num_engines);
    println!("Done!");
}
