//! The abstract layer
//!
//! Well for convenient, please add logging for c bind calls as debug.
//! Also whatever has to cast ptr under mut*

use crate::bindings::das::{
    das_context, das_context_eval_with_catch, das_context_find_function, das_context_get_exception,
    das_context_make, das_context_release, das_error_output, das_file_access,
    das_fileaccess_make_default, das_fileaccess_release, das_initialize, das_modulegroup_make,
    das_modulegroup_release, das_program, das_program_compile, das_program_context_stack_size,
    das_program_err_count, das_program_get_error, das_program_release, das_program_simulate,
    das_shutdown, das_text_make_printer, das_text_release, das_text_writer,
};
use log::{debug, error, info};
use std::{collections::HashMap, ffi::CString};

mod extended;
use extended::dasx_verif_fn;

pub struct VMEngine {
    das_fs: *mut das_file_access,
    das_tout: *mut das_text_writer,
    loaded_programs: HashMap<String, VMProgram>,
}

impl VMEngine {
    fn new() {
        unsafe {
            das_initialize();
        }
    }

    fn load(&mut self, path: &str) -> Option<&VMProgram> {
        if let Some(prog) = VMProgram::new(path) {
            self.loaded_programs.insert(path.to_string(), prog);
            self.loaded_programs.get(path)
        } else {
            None
        }
    }
}

impl Drop for VMEngine {
    fn drop(&mut self) {
        unsafe {
            das_fileaccess_release(self.das_fs);
            das_text_release(self.das_tout);
            das_shutdown();
        }
    }
}

/// The system to load a program and compile it, prepared for context hosting
pub struct VMProgram {
    program: *mut das_program,
}

impl VMProgram {
    /// Creates a new DaScriptExecutable from the given script path.
    pub fn new(script_path: &str) -> Option<Self> {
        let c_script_path = match CString::new(script_path) {
            Ok(s) => s,
            Err(_) => {
                error!("Invalid string path");
                return None;
            }
        };

        unsafe {
            debug!("VM: Creating file access");
            let ref_fs_access = das_fileaccess_make_default();
            if ref_fs_access.is_null() {
                error!("VM: Failed to create file access");
                return None;
            }

            debug!("VM: Creating text output");
            let tout = das_text_make_printer();
            if tout.is_null() {
                error!("VM: Failed to create text output");
                das_fileaccess_release(ref_fs_access);
                return None;
            }

            debug!("VM: Creating module group");
            let dummy_lib_group = das_modulegroup_make();
            if dummy_lib_group.is_null() {
                error!("VM: Failed to create module group");
                das_fileaccess_release(ref_fs_access);
                das_text_release(tout);
                return None;
            }

            debug!("VM: Compiling program: {}", script_path);
            let program = das_program_compile(
                c_script_path.as_ptr().cast_mut(),
                ref_fs_access,
                tout,
                dummy_lib_group,
            );

            // Check for compilation errors
            let err_count = if !program.is_null() {
                das_program_err_count(program)
            } else {
                0
            };

            if err_count > 0 {
                debug!("VM: Compilation failed with {} errors", err_count);
                for i in 0..err_count {
                    let error = das_program_get_error(program, i);
                    if !error.is_null() {
                        das_error_output(error, tout);
                    }
                }
            }

            debug!("VM: Finished compilation, releasing");
            das_fileaccess_release(ref_fs_access);
            das_modulegroup_release(dummy_lib_group);
            das_text_release(tout);

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
        VMContext::new(self.program)
    }
}

impl Drop for VMProgram {
    fn drop(&mut self) {
        unsafe {
            debug!("VM: Releasing program");
            das_program_release(self.program);
        }
    }
}

/// A context hosted from `VMProgram`, manage the context
pub struct VMContext {
    context: *mut das_context,
    tout: *mut das_text_writer,
}

impl VMContext {
    /// Creates a new VMContext
    fn new(program: *mut das_program) -> Option<Self> {
        unsafe {
            debug!("VM: Creating context");
            let context = das_context_make(das_program_context_stack_size(program));
            if context.is_null() {
                error!("VM: Failed to create context");
                return None;
            }

            debug!("VM: Creating text output");
            let tout = das_text_make_printer();
            if tout.is_null() {
                error!("VM: Failed to create text output");
                das_context_release(context);
                return None;
            }

            debug!("VM: Simulating program");
            if das_program_simulate(program, context, tout) == 0 {
                error!("VM: Simulation failed");
                let err_count = das_program_err_count(program);
                for i in 0..err_count {
                    let error = das_program_get_error(program, i);
                    if !error.is_null() {
                        das_error_output(error, tout);
                    }
                }
                das_context_release(context);
                das_text_release(tout);
                None
            } else {
                Some(VMContext { context, tout })
            }
        }
    }

    /// Find and evaluate a function by name
    pub fn eval_function(&self, name: &str) -> bool {
        if self.context.is_null() || self.tout.is_null() {
            error!("VM: Invalid context or text output");
            return false;
        }

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
            let function = das_context_find_function(self.context, c_name.as_ptr().cast_mut());
            if function.is_null() {
                error!("Function '{}' not found", name);
                return false;
            }

            debug!("EXT: Validate function pointer");
            if !dasx_verif_fn(function, c_name.into_raw()) {
                error!("Pointer is unsanitized");
                return false;
            }

            debug!("VM: Evaluating function with catch");
            let mut nullptr_allocated = [0f32, 0f32, 0f32, 0f32];
            das_context_eval_with_catch(
                self.context,
                function,
                nullptr_allocated.as_mut_ptr() as *mut _,
            );
            let exception = das_context_get_exception(self.context);
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
            das_context_release(self.context);
            das_text_release(self.tout);
        }
    }
}

#[no_mangle]
/// Initialize daScript runtime
pub extern "C" fn engine_initialize() {
    info!("VM: Initializing engine");
    unsafe {
        das_initialize();
    }
}

#[no_mangle]
/// Shutdown daScript runtime
pub extern "C" fn engine_shutdown() {
    info!("VM: Shutting down engine");
    unsafe {
        das_shutdown();
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
