use dastrap::interop::VMEngine;

fn main() {
    femme::with_level(log::LevelFilter::Debug);

    dastrap::interop::engine_initialize();

    // let program = VMProgram::new("examples/example.das").expect("Example failed: Failed to compile example.das");
    let mut engine = VMEngine::new().expect("VMEngine failed to initialize");
    
    let program = engine.load("examples/example.das").expect("Failed to load program");

    let context = program.host().expect("Example failed: Failed to host program.");

    context.eval_function("_not_exist");

    context.eval_function("test");

    context.eval_function("main");

    dastrap::interop::engine_shutdown();
}
