#[path = "../src/lib.rs"]
mod dastrap;
use dastrap::interop::VMProgram;

fn main() {
    femme::with_level(log::LevelFilter::Debug);

    dastrap::interop::engine_initialize();

    let program = VMProgram::new("examples/example.das").expect("Failed to compile example.das");

    let _context = program.host().expect("Failed to host program");

    dastrap::interop::engine_shutdown();
}
