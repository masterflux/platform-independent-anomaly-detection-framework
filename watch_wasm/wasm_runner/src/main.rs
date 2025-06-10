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
    // 1) Grab “watch_wasm.wasm” path from the command‐line
    let mut args = std::env::args().skip(1);
    let wasm_path = if let Some(p) = args.next() {
        PathBuf::from(p)
    } else {
        eprintln!("Usage: wasm_runner <path/to/watch_wasm.wasm>");
        std::process::exit(1);
    };

    // 2) Record wall-clock start
    let start_time = Instant::now();

    // 3) Build Wasmtime Engine & compile the module
    let mut config = Config::new();
    config.wasm_multi_memory(true).wasm_threads(true);
    let engine = Engine::new(&config)?;
    let module = Module::from_file(&engine, &wasm_path)?;

    // 4) Set up a WASI context with the current directory pre-opened
    let ambient = cap_std::ambient_authority();
    let dir = Dir::open_ambient_dir(".", ambient)?;
    let wasi_ctx = WasiCtxBuilder::new()
        .inherit_stdout()
        .inherit_stderr()
        .preopened_dir(dir, ".")?   // allow WASM to open files here
        .build();

    // 5) Create a Store whose data is our WasiCtx
    let mut store = Store::new(&engine, wasi_ctx);

    // 6) Create a Linker and push in the WASI functions
    let mut linker = Linker::new(&engine);
    add_to_linker(&mut linker, |ctx| ctx)?;

    // 7) Instantiate the module via the linker (satisfies WASI imports)
    let instance = linker.instantiate(&mut store, &module)?;

    // 8) Invoke the WASI entrypoint `_start`
    let wasm_start = instance
        .get_func(&mut store, "_start")
        .expect("`_start` not found in WASM");
    wasm_start.call(&mut store, &[], &mut [])?;

    // 9) WASM has finished—stop the timer
    let duration = start_time.elapsed();
    println!("---");
    println!("Wall-clock time: {:.3} s", duration.as_secs_f64());
    Ok(())
}