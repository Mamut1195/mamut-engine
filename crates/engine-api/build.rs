fn main() {
    println!("cargo:rerun-if-env-changed=MAMUT_ENGINE_GIT_COMMIT");
    println!("cargo:rerun-if-env-changed=TARGET");
    if let Ok(target) = std::env::var("TARGET") {
        println!("cargo:rustc-env=MAMUT_ENGINE_TARGET={target}");
    }
    // Solo una revisión hexadecimal explícita; no se invoca git ni se accede a red.
    if let Ok(commit) = std::env::var("MAMUT_ENGINE_GIT_COMMIT") {
        if matches!(commit.len(), 40 | 64) && commit.bytes().all(|b| b.is_ascii_hexdigit()) {
            println!("cargo:rustc-env=MAMUT_ENGINE_BUILD_COMMIT={commit}");
        }
    }
}
