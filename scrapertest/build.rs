fn main() {
 let triple = std::env::var("TARGET").unwrap();
 println!("cargo:rustc-link-search=rclone/build");
 println!("cargo:rustc-link-lib=rclone-{}", triple);
 println!("cargo:rerun-if-changed=../rclone/build/librclone-{}.a", triple);
}
