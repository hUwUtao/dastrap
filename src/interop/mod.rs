//! The abstract layer
//!
//! Well for convenient, please add logging for c bind calls as debug.
//! Also whatever has to cast ptr under mut*

/// all stdC generated bindings, includes `daScriptC.h`
pub mod c;

use c::{das_context, das_program};
use log::{debug, error, info};
use std::ffi::CString;

/// Compiled program
pub struct VMProgram {
    program: *mut das_program,
}

impl VMProgram {
    /// Creates a new DaScriptExecutable from the given script path.
    pub fn new(script_path: &str) -> Option<Self> {
        debug!("VM: Init program {}", script_path);
        let c_script_path = match CString::new(script_path) {
            Ok(s) => s,
            Err(_) => {
                error!("Invalid string path");
                return None;
            }
        };

        unsafe {
            debug!("VM: Creating file access");
            let ref_fs_access = c::das_fileaccess_make_default();
            if ref_fs_access.is_null() {
                error!("VM: Failed to create file access");
                return None;
            }

            debug!("VM: Creating text output");
            let tout = c::das_text_make_printer();
            if tout.is_null() {
                error!("VM: Failed to create text output");
                c::das_fileaccess_release(ref_fs_access);
                return None;
            }

            debug!("VM: Creating module group");
            let dummy_lib_group = c::das_modulegroup_make();
            if dummy_lib_group.is_null() {
                error!("VM: Failed to create module group");
                c::das_fileaccess_release(ref_fs_access);
                c::das_text_release(tout);
                return None;
            }

            debug!("VM: Compiling program: {}", script_path);
            let program = c::das_program_compile(
                c_script_path.as_ptr().cast_mut(),
                ref_fs_access,
                tout,
                dummy_lib_group,
            );

            // Check for compilation errors
            let err_count = if !program.is_null() {
                c::das_program_err_count(program)
            } else {
                0
            };

            if err_count > 0 {
                debug!("VM: Compilation failed with {} errors", err_count);
                for i in 0..err_count {
                    let error = c::das_program_get_error(program, i);
                    if !error.is_null() {
                        c::das_error_output(error, tout);
                    }
                }
            }

            debug!("VM: Releasing resources");
            c::das_fileaccess_release(ref_fs_access);
            c::das_modulegroup_release(dummy_lib_group);
            c::das_text_release(tout);

            if program.is_null() {
                error!("VM: Failed to compile program");
                None
            } else {
                Some(VMProgram { program })
            }
        }
    }

    /// Hosts the compiled program and returns a VMContext.
    pub fn host(&self) -> Option<VMContext> {
        unsafe {
            debug!("VM: Creating context");
            let context = c::das_context_make(c::das_program_context_stack_size(self.program));
            if context.is_null() {
                error!("VM: Failed to create context");
                return None;
            }

            debug!("VM: Creating text output");
            let tout = c::das_text_make_printer();
            if tout.is_null() {
                error!("VM: Failed to create text output");
                c::das_context_release(context);
                return None;
            }

            debug!("VM: Simulating program");
            if c::das_program_simulate(self.program, context, tout) == 0 {
                error!("VM: Simulation failed");
                let err_count = c::das_program_err_count(self.program);
                for i in 0..err_count {
                    let error = c::das_program_get_error(self.program, i);
                    if !error.is_null() {
                        c::das_error_output(error, tout);
                    }
                }
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
            debug!("VM: Releasing program");
            c::das_program_release(self.program);
        }
    }
}

/// Hosted context
pub struct VMContext {
    context: *mut das_context,
    tout: *mut c::das_text_writer,
}

impl VMContext {
    /// Find and evaluate a function by name
    pub fn eval_function(&self, name: &str) -> bool {
        debug!("VM: Evaluating function '{}'", name);
        unsafe {
            let c_name = match CString::new(name) {
                Ok(s) => s,
                Err(_) => {
                    error!("Invalid function name");
                    return false;
                }
            };
            debug!("VM: Finding function pointer");
            let function = c::das_context_find_function(self.context, c_name.as_ptr().cast_mut());
            if function.is_null() {
                error!("Function '{}' not found", name);
                return false;
            }

            debug!("VM: Evaluating function with catch");
            c::das_context_eval_with_catch(self.context, function, core::ptr::null_mut());
            debug!("VM: Checking for exceptions");
            let exception = c::das_context_get_exception(self.context);
            if !exception.is_null() {
                if let Ok(ex_str) = std::ffi::CStr::from_ptr(exception).to_str() {
                    error!("Exception while evaluating '{}': {}", name, ex_str);
                } else {
                    error!(
                        "Exception while evaluating '{}' (invalid UTF-8 in exception)",
                        name
                    );
                }
                return false;
            }
            debug!("VM: Function evaluation completed successfully");
            true
        }
    }
}

impl Drop for VMContext {
    fn drop(&mut self) {
        unsafe {
            debug!("VM: Releasing context");
            c::das_context_release(self.context);
            c::das_text_release(self.tout);
        }
    }
}

#[no_mangle]
/// Initialize daScript runtime
pub extern "C" fn engine_initialize() {
    info!("VM: Initializing engine");
    unsafe {
        c::das_initialize();
    }
}

#[no_mangle]
/// Shutdown daScript runtime
pub extern "C" fn engine_shutdown() {
    info!("VM: Shutting down engine");
    unsafe {
        c::das_shutdown();
    }
}

#[no_mangle]
/// Load program into context
pub extern "C" fn engine_load_program(
    path: *const std::os::raw::c_char,
    len: usize,
) -> *mut VMProgram {
    if path.is_null() {
        error!("Null path provided");
        return std::ptr::null_mut();
    }

    debug!("Loading program from raw path");
    let script_path = unsafe {
        match std::str::from_utf8(std::slice::from_raw_parts(path as *const u8, len)) {
            Ok(s) => s,
            Err(_) => {
                error!("Invalid UTF-8 in path");
                return std::ptr::null_mut();
            }
        }
    };

    let executable = VMProgram::new(script_path);
    match executable {
        Some(exe) => Box::into_raw(Box::new(exe)),
        None => std::ptr::null_mut(),
    }
}
