use dastrap::interop::VMProgram;

fn main() {
    femme::with_level(log::LevelFilter::Debug);

    dastrap::interop::engine_initialize();

    let program = VMProgram::new("examples/example.das").expect("Failed to compile example.das");

    let context = program.host().expect("Failed to host program");

    context.eval_function("main");

    dastrap::interop::engine_shutdown();
}
