# watch_wasm

A Rust-based change point detection toolkit compiled to WebAssembly (WASI) for easy deployment and integration.

## Prerequisites

- **Rust**: Install via [rustup](https://rustup.rs/)
- **WASI target**: `wasm32-wasip1`
- **WASM runtime**: [Wasmtime CLI](https://github.com/bytecodealliance/wasmtime)

## Setup

1. **Install Rust**

   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://rustup.rs | sh
   
2. Add the WASI target
   
   ```bash
   rustup target add wasm32-wasip1
   
3. Install Wasmtime CLI
   
   ```bash
   cargo install wasmtime-cli

4. Creating the Project

   ```bash
   cargo new watch_wasm --bin
   cd watch_wasm
   ```
5. Building for WASI

   Compile your Rust code to WASM with release optimizations:

   ```bash
   cargo build --target wasm32-wasip1 --release
   ```

6. Running the WASM Module

   Execute the generated WebAssembly binary with Wasmtime:

   ```bash
   # Basic run
   wasmtime target/wasm32-wasip1/release/watch_wasm.wasm
    
   # Grant read access to current directory (e.g., to load input.csv)
   wasmtime run --dir . target/wasm32-wasip1/release/watch_wasm.wasm
   ```

   Project Structure 

   ```bash
   watch_wasm/
   ├── Cargo.toml
   └── src
       ├── main.rs               # Entry point, orchestrates all detectors
       ├── lib.rs                # Library exports
       ├── utils.rs              # Helper functions (CSV loading, gamma, erf)
       ├── distance_measures.rs  # All distance metric implementations
       ├── change_point_detector.rs # Base trait definition
       └── detectors
           ├── mod.rs            # Detector module declarations
           ├── bocpd.rs          # Bayesian Online Change Point Detection
           ├── cusum.rs          # Cumulative Sum Detector
           ├── micro_watch.rs    # Micro-Watch Batch Detector
           ├── pelt.rs           # Pruned Exact Linear Time Detector
           └── bocpdms.rs        # Simplified Multivariate BOCPD
    ```

  License
  This project is licensed under the MIT License. Feel free to use and modify as needed.


