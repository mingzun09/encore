fn main() {
    let target = std::env::var("TARGET").unwrap_or_default();
    if target.contains("android") {
        println!("cargo:rustc-link-lib=dylib=binder_ndk");
        println!("cargo:rustc-link-lib=dylib=log");
    }
}
