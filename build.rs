use cmake::Config;

fn main() {
    let dastrap_dst = Config::new(".")
        .define("CMAKE_MSVC_RUNTIME_LIBRARY", "MultiThreaded")
        .profile("RelWithDebInfo")
        .build();

    let dascript_dst = Config::new("./libs/daScript")
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
        .define("DAS_ENABLE_EXCEPTIONS", "0")
        .define("CMAKE_MSVC_RUNTIME_LIBRARY", "MultiThreaded")
        .profile("RelWithDebInfo")
        .build();

    setup_linking(&dascript_dst, &dastrap_dst);
}

fn setup_linking(dascript_dst: &std::path::Path, dastrap_dst: &std::path::Path) {
    if std::path::Path::new("build/Release").exists() {
        println!("cargo:rustc-link-search=native=build/Release");
    }
    if std::path::Path::new("build/RelWithDebInfo").exists() {
        println!("cargo:rustc-link-search=native=build/RelWithDebInfo");
    }
    if std::path::Path::new("libs/daScript/cmake_temp/Release").exists() {
        println!("cargo:rustc-link-search=native=libs/daScript/cmake_temp/Release");
    }
    if std::path::Path::new("libs/daScript/cmake_temp/RelWithDebInfo").exists() {
        println!("cargo:rustc-link-search=native=libs/daScript/cmake_temp/RelWithDebInfo");
    }

    println!(
        "cargo:rustc-link-search=native={}/build/Release",
        dascript_dst.display()
    );
    println!(
        "cargo:rustc-link-search=native={}/build/RelWithDebInfo",
        dascript_dst.display()
    );
    println!(
        "cargo:rustc-link-search=native={}/build",
        dascript_dst.display()
    );
    println!(
        "cargo:rustc-link-search=native={}/build",
        dastrap_dst.display()
    );

    println!("cargo:rustc-link-lib=static=libDaScript");
    println!("cargo:rustc-link-lib=static=libDaStrap");

    #[cfg(all(target_os = "windows", target_env = "msvc"))]
    {
        println!("cargo:rustc-link-lib=msvcrt");
        println!("cargo:rustc-link-lib=vcruntime");
        println!("cargo:rustc-link-lib=ucrt");
    }
}
