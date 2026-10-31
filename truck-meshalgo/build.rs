#[cfg(feature = "cuda")]
fn find_compatible_ccbin() -> Option<&'static str> {
    for bin in &["g++-13", "g++-12", "clang++-18", "clang++-17"] {
        if std::process::Command::new(bin).arg("--version").output().is_ok() {
            return Some(bin);
        }
    }
    None
}

fn main() {
    println!("cargo:rerun-if-changed=src/gpu/kernels.cu");

    #[cfg(feature = "cuda")]
    {
        if std::process::Command::new("nvcc").arg("--version").output().is_ok() {
            let mut build = cc::Build::new();
            build
                .cuda(true)
                .flag("-arch=sm_86")
                .flag("-O3")
                .flag("-Xcompiler")
                .flag("-fPIC")
                .file("src/gpu/kernels.cu");

            if let Some(ccbin) = find_compatible_ccbin() {
                std::env::set_var("CXX", ccbin);
            }

            build.compile("truck_cuda");

            println!("cargo:rustc-link-lib=cudart");
            if let Ok(cuda_path) = std::env::var("CUDA_PATH") {
                println!("cargo:rustc-link-search=native={}/lib64", cuda_path);
            } else {
                println!("cargo:rustc-link-search=native=/usr/local/cuda/lib64");
                println!("cargo:rustc-link-search=native=/usr/local/cuda/targets/x86_64-linux/lib");
                println!("cargo:rustc-link-search=native=/usr/lib/x86_64-linux-gnu");
            }
        }
    }
}
