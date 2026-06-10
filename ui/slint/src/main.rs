use slint_app::run;

#[cfg(not(any(target_os = "windows", target_os = "android")))]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

#[cfg(not(target_os = "android"))]
#[tokio::main]
async fn main() { run().await; }

#[cfg(target_os = "android")]
fn main() {}
