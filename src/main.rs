mod constants;

#[cfg(target_os = "windows")]
mod windows;


#[tokio::main]
async fn main() {
    #[cfg(target_os = "windows")]
    let _ = windows::run().await;

    #[cfg(not(target_os = "windows"))]
    println!("Not supported on this OS");
}
