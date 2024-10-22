mod c;

use c::{das_context, das_program};
use std::ffi::CString;

pub struct DaScriptExecutable {
    program: *mut das_program,
}

impl DaScriptExecutable {
    /// Creates a new DaScriptExecutable from the given script path.
    pub fn new(script_path: &str) -> Option<Self> {
        let c_script_path = CString::new(script_path).ok()?;

        unsafe {
            let fAccess = c::das_fileaccess_make_default();
            let tout = c::das_text_make_printer();
            let dummyLibGroup = c::das_modulegroup_make();

            let program = c::das_program_compile(
                c_script_path.as_ptr() as *mut _,
                fAccess,
                tout,
                dummyLibGroup,
            );

            c::das_fileaccess_release(fAccess);
            c::das_text_release(tout);
            c::das_modulegroup_release(dummyLibGroup);

            if program.is_null() {
                None
            } else {
                Some(DaScriptExecutable { program })
            }
        }
    }

    /// Hosts the compiled program and returns a context.
    pub fn host(&self) -> Option<*mut das_context> {
        unsafe {
            let context = c::das_context_make(c::das_program_context_stack_size(self.program));
            let tout = c::das_text_make_printer();

            if c::das_program_simulate(self.program, context, tout) == 0 {
                c::das_context_release(context);
                c::das_text_release(tout);
                None
            } else {
                c::das_text_release(tout);
                Some(context)
            }
        }
    }
}

impl Drop for DaScriptExecutable {
    fn drop(&mut self) {
        unsafe {
            c::das_program_release(self.program);
        }
    }
}
