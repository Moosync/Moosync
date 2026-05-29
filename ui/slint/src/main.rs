use slint_app::run;

#[cfg(not(target_os = "android"))]
#[tokio::main]
async fn main() {
    run().await;
}

#[cfg(target_os = "android")]
fn main() {}
