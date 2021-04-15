use criterion::*;
use std::path::Path;
use std::time::Instant;
use wasi_common::WasiFile;

fn wasi_engine(strategy: wasmtime::InstanceAllocationStrategy) -> wasmtime::Engine {
    let mut config = wasmtime::Config::new();

    config.cache_config_load_default().unwrap();
    config.allocation_strategy(strategy);
    // config.profiler(wasmtime::ProfilingStrategy::JitDump).unwrap();
    // config.debug_info(true);

    wasmtime_wasi::Wasi::add_to_config(&mut config);
    wasmtime::Engine::new(&config).unwrap()
}

fn make_wasi_ctx(
    stdin: Option<Box<dyn WasiFile>>,
    dirs: &[(&Path, &Path)],
) -> wasmtime_wasi::WasiCtx {
    let mut wasi_ctx = wasi_cap_std_sync::WasiCtxBuilder::new()
        // .inherit_stdio()
        .arg("spelling-corrector.wasm")
        .unwrap()
        .arg("funciton")
        .unwrap();
    if let Some(stdin) = stdin {
        wasi_ctx = wasi_ctx.stdin(stdin);
    }
    for (host_dir, guest_dir) in dirs {
        let preopened = unsafe { cap_std::fs::Dir::open_ambient_dir(host_dir).unwrap() };
        wasi_ctx = wasi_ctx.preopened_dir(preopened, guest_dir).unwrap();
    }
    wasi_ctx.build().unwrap()
}

fn wasi_linker(
    engine: &wasmtime::Engine,
    stdin: Option<Box<dyn WasiFile>>,
    dirs: &[(&Path, &Path)],
) -> wasmtime::Linker {
    let store = wasmtime::Store::new(&engine);
    wasmtime_wasi::Wasi::set_context(&store, make_wasi_ctx(stdin, dirs))
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

    let engine = wasi_engine(if cfg!(feature = "uffd") {
        wasmtime::InstanceAllocationStrategy::Pooling {
            strategy: wasmtime::PoolingAllocationStrategy::NextAvailable,
            module_limits: wasmtime::ModuleLimits {
                memory_pages: 554,
                ..Default::default()
            },
            instance_limits: Default::default(),
        }
    } else {
        wasmtime::InstanceAllocationStrategy::OnDemand
    });

    group.bench_function("control", |b| {
        let wasm = std::fs::read("../spelling-corrector/control.wasm").unwrap();
        let module = wasmtime::Module::new(&engine, &wasm).unwrap();
        b.iter(|| {
            let linker = wasi_linker(
                &engine,
                None,
                &[(Path::new("../spelling-corrector"), Path::new("."))],
            );
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
            let linker = wasi_linker(&engine, None, &[]);
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
                memory_pages: 18,
                ..Default::default()
            },
            instance_limits: Default::default(),
        }
    } else {
        wasmtime::InstanceAllocationStrategy::OnDemand
    });

    group.bench_function("instantiate", |b| {
        let wasm = std::fs::read("../markdown/markdown.wasm").unwrap();
        let module = wasmtime::Module::new(&engine, &wasm).unwrap();
        b.iter_custom(|iters| {
            let mut elapsed = Default::default();
            for _ in 0..iters {
                let linker = wasi_linker(&engine, None, &[]);
                let start = Instant::now();
                for _ in 0..RUNS {
                    let instance = linker.instantiate(&module).unwrap();
                    criterion::black_box(instance);
                }
                elapsed += start.elapsed();
            }
            elapsed
        });
    });
}

fn js(c: &mut Criterion) {
    const RUNS: u64 = 1;

    let mut group = c.benchmark_group("js");
    group.throughput(Throughput::Elements(RUNS));

    let engine = wasi_engine(if cfg!(feature = "uffd") {
        wasmtime::InstanceAllocationStrategy::Pooling {
            strategy: wasmtime::PoolingAllocationStrategy::NextAvailable,
            module_limits: wasmtime::ModuleLimits {
                memory_pages: 84,
                types: 104,
                ..Default::default()
            },
            instance_limits: Default::default(),
        }
    } else {
        wasmtime::InstanceAllocationStrategy::OnDemand
    });

    group.bench_function("control", |b| {
        let wasm = std::fs::read("../js-demo/js.wasm").unwrap();
        let module = wasmtime::Module::new(&engine, &wasm).unwrap();
        let stdin = include_str!("../../js-demo/dist/main.js");
        b.iter(|| {
            let stdin = wasi_common::pipe::ReadPipe::from(stdin);
            let linker = wasi_linker(&engine, Some(Box::new(stdin) as _), &[]);
            for _ in 0..RUNS {
                let instance = linker.instantiate(&module).unwrap();
                call_start(&instance);
            }
        });
    });

    group.bench_function("wizer", |b| {
        let wasm = std::fs::read("../js-demo/dist/main.js.wasm").unwrap();
        let module = wasmtime::Module::new(&engine, &wasm).unwrap();
        b.iter(|| {
            let linker = wasi_linker(&engine, None, &[]);
            for _ in 0..RUNS {
                let instance = linker.instantiate(&module).unwrap();
                call_start(&instance);
            }
        });
    });

    group.bench_function("control-instantiate", |b| {
        let wasm = std::fs::read("../js-demo/js.wasm").unwrap();
        let module = wasmtime::Module::new(&engine, &wasm).unwrap();
        b.iter(|| {
            let linker = wasi_linker(&engine, None, &[]);
            for _ in 0..RUNS {
                let instance = linker.instantiate(&module).unwrap();
                black_box(instance);
            }
        });
    });

    group.bench_function("wizer-instantiate", |b| {
        let wasm = std::fs::read("../js-demo/dist/main.js.wasm").unwrap();
        let module = wasmtime::Module::new(&engine, &wasm).unwrap();
        b.iter(|| {
            let linker = wasi_linker(&engine, None, &[]);
            for _ in 0..RUNS {
                let instance = linker.instantiate(&module).unwrap();
                black_box(instance);
            }
        });
    });
}

criterion_group!(benches, spelling_corrector, instantiate, js);
criterion_main!(benches);
