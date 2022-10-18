use std::cell::RefCell;
use wasmtime::*;
use wasmtime_wasi::WasiCtx;

#[derive(Clone)]
pub enum ModuleResult {
    None,
    ByteArray(Vec<u8>),
    String(String),
}

thread_local! {
    pub static CURRENT_RESULT: RefCell<ModuleResult> = RefCell::new(ModuleResult::None);
}

// copy bytearray out of sandbox
pub fn server_module_bytearr_result(mut caller: Caller<'_, WasiCtx>, string_resp: u32, bytes: u32) {
    let mem = match caller.get_export("memory") {
        Some(Extern::Memory(mem)) => mem,
        _ => panic!("failed to find host memory"),//return Err(Trap::new("failed to find host memory")),
    };
    let response_slice = mem.data(&caller)
        .get(string_resp as usize..)
        .and_then(|arr| arr.get(..bytes as usize)).unwrap();

    let v = response_slice.to_vec();

    CURRENT_RESULT.with(|current_result| {
        *current_result.borrow_mut() = ModuleResult::ByteArray(v);
    });
}

// copy string out of sandbox
pub fn server_module_string_result(mut caller: Caller<'_, WasiCtx>, string_resp: u32, bytes: u32) {
    let mem = match caller.get_export("memory") {
        Some(Extern::Memory(mem)) => mem,
        _ => panic!("failed to find host memory"),//return Err(Trap::new("failed to find host memory")),
    };
    let response_slice = mem.data(&caller)
        .get(string_resp as usize..)
        .and_then(|arr| arr.get(..bytes as usize)).unwrap();

    let v = response_slice.to_vec();

    let str_buf: String = unsafe { String::from_utf8_unchecked(v) };
    CURRENT_RESULT.with(|current_result| {
        *current_result.borrow_mut() = ModuleResult::String(str_buf);
    });
}


pub fn benchmark_name<'a>(strategy: &InstanceAllocationStrategy) -> &'static str {
    match strategy {
        InstanceAllocationStrategy::OnDemand => "default",
        InstanceAllocationStrategy::Pooling { .. } => "pooling",
    }
}
