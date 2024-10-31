use cmake::Config;

fn main() {
    // println!("cargo:rerun-if-changed=src/interop/extended");
    println!("cargo:rerun-if-changed=libs/daScript");
    
    macro_rules! add_search_path {
        ($path:expr) => {
            println!("cargo:rustc-link-search=native={}", $path.display())
        };
    }

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
        .profile("RelWithDebInfo")
        .build_target("libDaScript")
        .build();
    
    add_search_path!(&dascript_dst.join("build/Release"));
    add_search_path!(&dascript_dst.join("build/RelWithDebInfo"));
    add_search_path!(&dascript_dst.join("build"));

    println!("cargo:rustc-link-lib=static=libDaScript");

    // let dastrap_dst = Config::new(".")
    //     .profile("RelWithDebInfo")
    //     .build_target("libDaStrap")
    //     .build();

    // add_search_path!(&dastrap_dst.join("lib"));
    // add_search_path!(&dastrap_dst.join("build/Release"));
    // add_search_path!(&dastrap_dst.join("build/RelWithDebInfo"));
    // add_search_path!(&dastrap_dst.join("build"));
    
    // println!("cargo:rustc-link-lib=static=libDaStrap");
}