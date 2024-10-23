#[allow(clippy::all)]
mod c;

use c::{das_context, das_program};
use std::ffi::CString;

pub struct VMProgram {
    program: *mut das_program,
}

impl VMProgram {
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
            c::das_modulegroup_release(dummyLibGroup);
            c::das_text_release(tout);

            if program.is_null() {
                None
            } else {
                Some(VMProgram { program })
            }
        }
    }

    /// Hosts the compiled program and returns a VMContext.
    pub fn host(&self) -> Option<VMContext> {
        unsafe {
            let context = c::das_context_make(c::das_program_context_stack_size(self.program));
            let tout = c::das_text_make_printer();

            if c::das_program_simulate(self.program, context, tout) == 0 {
                c::das_context_release(context);
                c::das_text_release(tout);
                None
            } else {
                Some(VMContext { context, tout })
            }
        }
    }
}

impl Drop for VMProgram {
    fn drop(&mut self) {
        unsafe {
            c::das_program_release(self.program);
        }
    }
}

pub struct VMContext {
    context: *mut das_context,
    tout: *mut c::das_text_writer,
}

impl Drop for VMContext {
    fn drop(&mut self) {
        unsafe {
            c::das_context_release(self.context);
            c::das_text_release(self.tout);
        }
    }
}

#[no_mangle]
pub extern "C" fn load_program(path: *const std::os::raw::c_char, len: usize) -> *mut VMProgram {
    let script_path = unsafe {
        std::str::from_utf8_unchecked(std::slice::from_raw_parts(path as *const u8, len))
    };
    let executable = VMProgram::new(script_path);
    match executable {
        Some(exe) => Box::into_raw(Box::new(exe)),
        None => std::ptr::null_mut(),
    }
}
