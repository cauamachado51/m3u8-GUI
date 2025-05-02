fn main() {
    println!("cargo:rerun-if-changed=icon.res");
    println!("cargo:rustc-link-arg-bin=m3u8-GUI=icon.res");
}
