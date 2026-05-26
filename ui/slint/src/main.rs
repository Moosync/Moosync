#[cfg(not(target_os = "android"))]
#[tokio::main]
async fn main() {
    slint_app::run();
}

#[cfg(target_os = "android")]
fn main() {}
