use cmake::Config;

fn main() {
    // let out_path = PathBuf::from("src/interop");
    // let binding_file = out_path.join("c.rs");

    // if env::var("PLS_NEW_BINDING").is_ok() || !binding_file.exists() {
    //     // Generate bindings for daScriptC.h
    //     let bindings_c = Builder::default()
    //         .header("./libs/daScript/include/daScript/daScriptC.h")
    //         .clang_arg("-I./libs/libDaScript/include")
    //         .generate()
    //         .expect("Unable to generate bindings for daScriptC.h");

    //     bindings_c
    //         .write_to_file(&binding_file)
    //         .expect("Couldn't write bindings for daScriptC.h!");
    // }

    // Generate bindings for daScript.h
    // let bindings = Builder::default()
    //     .header("./libs/libDaScript/include/daScript/daScript.h")
    //     .clang_arg("-I./libs/libDaScript/include")
    //     .generate()
    //     .expect("Unable to generate bindings for daScript.h");

    // bindings
    //     .write_to_file(out_path.join("bindings.rs"))
    //     .expect("Couldn't write bindings for daScript.h!");

    let dst = Config::new("./libs/daScript")
        .define("DAS_CLANG_BIND_DISABLED", "ON")
        .define("DAS_LLVM_DISABLED", "ON")
        .define("DAS_QUIRREL_DISABLED", "ON")
        .define("DAS_HV_DISABLED", "ON")
        .define("DAS_IMGUI_DISABLED", "ON")
        .define("DAS_BGFX_DISABLED", "ON")
        .define("DAS_XBYAK_DISABLED", "ON")
        .define("DAS_MINFFT_DISABLED", "ON")
        .define("DAS_AUDIO_DISABLED", "ON")
        .define("DAS_SFML_DISABLED", "ON")
        .define("DAS_PUGIXML_DISABLED", "ON")
        .define("DAS_SQLITE_DISABLED", "ON")
        .define("DAS_PROFILE_DISABLED", "ON")
        .define("DAS_TESTS_DISABLED", "ON")
        .define("DAS_GLFW_DISABLED", "ON")
        .define("DAS_STDDLG_DISABLED", "ON")
        .define("DAS_STBIMAGE_DISABLED", "ON")
        .define("DAS_STBTRUETYPE_DISABLED", "ON")
        .define("DAS_TOOLS_DISABLED", "ON")
        .define("DAS_AOT_EXAMPLES_DISABLED", "ON")
        .define("DAS_TUTORIAL_DISABLED", "ON")
        .profile("Release")
        .build();
    // Link search paths
    println!(
        "cargo:rustc-link-search=native={}/build/Debug",
        dst.display()
    );
    println!(
        "cargo:rustc-link-search=native={}/build/Release",
        dst.display()
    );
    println!("cargo:rustc-link-search=native={}/build", dst.display());

    // Libraries
    println!("cargo:rustc-link-lib=static=libDaScript");

    // Windows-specific linking for MSVC target
    #[cfg(all(target_os = "windows", target_env = "msvc"))]
    {
        println!("cargo:rustc-link-lib=msvcrt");
        println!("cargo:rustc-link-lib=vcruntime");
        println!("cargo:rustc-link-lib=ucrt");
    }
}
