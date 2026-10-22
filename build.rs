fn main() {
    println!("cargo:rerun-if-changed=native/telemetry.cpp");
    cc::Build::new()
        .cpp(true)
        .file("native/telemetry.cpp")
        .flag("-std=c++17")
        .compile("rs_telemetry");
}
