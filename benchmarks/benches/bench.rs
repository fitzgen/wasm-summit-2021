use criterion::*;
use std::path::Path;

fn wasi_engine(strategy: wasmtime::InstanceAllocationStrategy) -> wasmtime::Engine {
    let mut config = wasmtime::Config::new();

    config.allocation_strategy(strategy);

    wasmtime_wasi::Wasi::add_to_config(&mut config);
    wasmtime::Engine::new(&config).unwrap()
}

fn make_wasi_ctx(dirs: &[(&Path, &Path)]) -> wasmtime_wasi::WasiCtx {
    let mut wasi_ctx = wasi_cap_std_sync::WasiCtxBuilder::new()
        // .inherit_stdio()
        .arg("spelling-corrector.wasm")
        .unwrap()
        .arg("funciton")
        .unwrap();
    for (host_dir, guest_dir) in dirs {
        let preopened = unsafe { cap_std::fs::Dir::open_ambient_dir(host_dir).unwrap() };
        wasi_ctx = wasi_ctx.preopened_dir(preopened, guest_dir).unwrap();
    }
    wasi_ctx.build().unwrap()
}

fn wasi_linker(engine: &wasmtime::Engine) -> wasmtime::Linker {
    let store = wasmtime::Store::new(&engine);
    wasmtime_wasi::Wasi::set_context(
        &store,
        make_wasi_ctx(&[(Path::new("../spelling-corrector"), Path::new("."))]),
    )
    .ok()
    .unwrap();
    wasmtime::Linker::new(&store)
}

fn call_start(instance: &wasmtime::Instance) {
    let f = instance.get_typed_func::<(), ()>("_start").unwrap();
    if let Err(trap) = f.call(()) {
        match trap.i32_exit_status() {
            Some(0) => return,
            Some(n) => panic!("_start exited with a non-zero code: {}", n),
            None => panic!("executing the benchmark resulted in a trap: {}", trap),
        }
    }
}

fn spelling_corrector(c: &mut Criterion) {
    const RUNS: u64 = 10;

    let mut group = c.benchmark_group("spelling-corrector");
    group.throughput(Throughput::Elements(RUNS));

    let engine = wasi_engine(wasmtime::InstanceAllocationStrategy::OnDemand);

    group.bench_function("control", |b| {
        let wasm = std::fs::read("../spelling-corrector/control.wasm").unwrap();
        let module = wasmtime::Module::new(&engine, &wasm).unwrap();
        b.iter(|| {
            let linker = wasi_linker(&engine);
            for _ in 0..RUNS {
                let instance = linker.instantiate(&module).unwrap();
                call_start(&instance);
            }
        });
    });

    group.bench_function("wizer", |b| {
        let wasm = std::fs::read("../spelling-corrector/wizer.wasm").unwrap();
        let module = wasmtime::Module::new(&engine, &wasm).unwrap();
        b.iter(|| {
            let linker = wasi_linker(&engine);
            for _ in 0..RUNS {
                let instance = linker.instantiate(&module).unwrap();
                call_start(&instance);
            }
        });
    });
}

// NB: test this with vs without the `uffd` feature enabled.
fn instantiate(c: &mut Criterion) {
    // Any higher will require updating the instance limits.
    const RUNS: u64 = 1_000;

    let mut group = c.benchmark_group("instantiate");
    group.throughput(Throughput::Elements(RUNS));

    let engine = wasi_engine(if cfg!(feature = "uffd") {
        wasmtime::InstanceAllocationStrategy::Pooling {
            strategy: wasmtime::PoolingAllocationStrategy::NextAvailable,
            module_limits: wasmtime::ModuleLimits {
                memory_pages: 17,
                ..Default::default()
            },
            instance_limits: Default::default(),
        }
    } else {
        wasmtime::InstanceAllocationStrategy::OnDemand
    });

    group.bench_function("instantiate", |b| {
        let wasm = std::fs::read("../hello/hello.wasm").unwrap();
        let module = wasmtime::Module::new(&engine, &wasm).unwrap();
        b.iter(|| {
            let linker = wasi_linker(&engine);
            for _ in 0..RUNS {
                let instance = linker.instantiate(&module).unwrap();
                criterion::black_box(instance);
            }
        });
    });
}

criterion_group!(benches, spelling_corrector, instantiate);
criterion_main!(benches);
