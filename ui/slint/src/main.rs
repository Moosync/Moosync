#[cfg(target_os = "linux")]
use prctl::set_thp_disable;
use slint_app::run;

#[cfg(not(any(target_os = "windows", target_os = "android")))]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

#[cfg(not(target_os = "android"))]
fn main() {
    #[cfg(target_os = "linux")]
    let _ = set_thp_disable(true);

    run();
}

#[cfg(target_os = "android")]
fn main() {}
