use std::thread::available_parallelism;

#[cfg(target_os = "linux")]
use prctl::set_thp_disable;
use slint_app::run;
use tokio::runtime::Builder;

#[cfg(not(any(target_os = "windows", target_os = "android")))]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

#[cfg(not(target_os = "android"))]
fn main() {
    #[cfg(target_os = "linux")]
    let _ = set_thp_disable(true);

    let threads = available_parallelism()
        .map(|parallelism| parallelism.get().clamp(2, 4))
        .unwrap_or(4);

    let runtime = Builder::new_multi_thread()
        .worker_threads(threads)
        .enable_all()
        .build()
        .expect("Failed to build Tokio runtime");

    runtime.block_on(run());
}

#[cfg(target_os = "android")]
fn main() {}
