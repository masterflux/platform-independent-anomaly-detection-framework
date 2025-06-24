use std::{
    path::PathBuf,
    time::Instant,
};
use anyhow::Result;
use cap_std::fs::Dir;
use wasmtime::{Config, Engine, Linker, Module, Store};
use wasmtime_wasi::{add_to_linker, WasiCtxBuilder};

type RunnerResult<T> = std::result::Result<T, anyhow::Error>;

fn main() -> RunnerResult<()> {
    
    let mut args = std::env::args().skip(1);
    let wasm_path = if let Some(p) = args.next() {
        PathBuf::from(p)
    } else {
        eprintln!("Usage: wasm_runner <path/to/watch_wasm.wasm>");
        std::process::exit(1);
    };

    
    let start_time = Instant::now();

    
    let mut config = Config::new();
    config.wasm_multi_memory(true).wasm_threads(true);
    let engine = Engine::new(&config)?;
    let module = Module::from_file(&engine, &wasm_path)?;

    
    let ambient = cap_std::ambient_authority();
    let dir = Dir::open_ambient_dir(".", ambient)?;
    let wasi_ctx = WasiCtxBuilder::new()
        .inherit_stdout()
        .inherit_stderr()
        .preopened_dir(dir, ".")?   
        .build();

    
    let mut store = Store::new(&engine, wasi_ctx);

    
    let mut linker = Linker::new(&engine);
    add_to_linker(&mut linker, |ctx| ctx)?;

    
    let instance = linker.instantiate(&mut store, &module)?;

    
    let wasm_start = instance
        .get_func(&mut store, "_start")
        .expect("`_start` not found in WASM");
    wasm_start.call(&mut store, &[], &mut [])?;

    
    let duration = start_time.elapsed();
    println!("---");
    println!("Wall-clock time: {:.3} s", duration.as_secs_f64());
    Ok(())
}