//! Allow only Linux build
fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap() != "linux" {
        panic!("This application can be built on Linux only");
    }
    #[cfg(feature = "gui")]
    slint_build::compile("src/ui/ui.slint").expect("Slint build failed");
}
