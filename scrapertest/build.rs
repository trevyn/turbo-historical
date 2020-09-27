fn main() {
 println!("cargo:rustc-link-search=rclone/build");
 println!("cargo:rustc-link-lib=rclone-x86_64-apple-darwin");
 println!("cargo:rerun-if-changed=../rclone/build/librclone-x86_64-apple-darwin.a");
}
