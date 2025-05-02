fn main() {
    #[cfg(target_os = "windows")]
    {
        println!("cargo:rerun-if-changed=resources\\icon.res");
        println!("cargo:rustc-link-arg-bin=m3u8-GUI=resources\\icon.res");
    }
}