fn main() {
    println!("cargo:rerun-if-changed=native/telemetry.cpp");
    cc::Build::new()
        .cpp(true)
        .file("native/telemetry.cpp")
        .flag("-std=c++17")
        .compile("rs_telemetry");
    println!("cargo:rerun-if-env-changed=MLX_ROOT");
    if std::env::var_os("CARGO_FEATURE_NATIVE_MLX").is_some() {
        let root = std::env::var_os("MLX_ROOT")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::path::PathBuf::from("vendor/mlx"));
        let root = root
            .canonicalize()
            .expect("install native MLX SDK in vendor/mlx or set MLX_ROOT");
        assert!(
            root.join("include/mlx/mlx.h").is_file(),
            "MLX headers missing"
        );
        assert!(
            root.join("lib/libmlx.dylib").is_file(),
            "native MLX library missing"
        );
        println!("cargo:rerun-if-changed=native/mlx_bridge.cpp");
        println!("cargo:rerun-if-changed=native/mlx_bridge.h");
        println!("cargo:rerun-if-changed=native/qwen_model.inc");
        for item in [
            "include",
            "lib/libmlx.dylib",
            "lib/libjaccl.dylib",
            "lib/mlx.metallib",
        ] {
            println!("cargo:rerun-if-changed={}", root.join(item).display());
        }
        println!(
            "cargo:rerun-if-changed={}",
            root.join("include/mlx/version.h").display()
        );
        cc::Build::new()
            .cpp(true)
            .file("native/mlx_bridge.cpp")
            .include(root.join("include"))
            .flag("-std=c++20")
            .compile("rs_mlx_bridge");
        println!(
            "cargo:rustc-link-search=native={}",
            root.join("lib").display()
        );
        println!("cargo:rustc-link-lib=dylib=mlx");
        println!(
            "cargo:rustc-link-arg=-Wl,-rpath,{}",
            root.join("lib").display()
        );
    }
}
