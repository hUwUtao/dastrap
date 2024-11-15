//! The abstract layer
//!
//! Well for convenient, please add logging for c bind calls as debug.
//! Also whatever has to cast ptr under mut*

use crate::bindings::das::{
    das_context, das_context_eval_with_catch_unaligned, das_context_find_function,
    das_context_get_exception, das_context_make, das_context_release, das_error_output,
    das_file_access, das_fileaccess_make_default, das_fileaccess_release, das_initialize,
    das_module_group, das_modulegroup_make, das_modulegroup_release, das_program,
    das_program_compile, das_program_context_stack_size, das_program_err_count,
    das_program_get_error, das_program_release, das_program_simulate, das_shutdown,
    das_text_make_printer, das_text_release, das_text_writer, V4FloatUnlined,
};
use log::{debug, error, info};
use parking_lot::RwLock;
use std::{collections::HashMap, ffi::CString, sync::Arc};

// mod extended;
// use extended::dasx_verif_fn;

/// `VMEngine` must flush the item before dying
pub trait VMHang: Sized {}

/// Track for contexes, which allows graceful shutdown
pub struct VMState {
    hanged: bool,
    tracked: Vec<VMHangedLock<VMContext>>,
}

/// Hanging object, which allows to drop but no
pub struct VMHanging<T: VMHang> {
    destroyed: bool,
    hanged: Option<Box<T>>,
}

/// Engine, the host of dascript
pub struct VMEngine {
    das_fs: *mut das_file_access,
    das_tout: *mut das_text_writer,
    das_libs: *mut das_module_group,
    state: Arc<RwLock<VMState>>,
    sys_progs: HashMap<String, Arc<VMProgram>>,
}

impl VMEngine {
    pub fn new() -> Option<Self> {
        unsafe {
            das_initialize();

            debug!("VM: Creating file access");
            let das_fs = das_fileaccess_make_default();
            if das_fs.is_null() {
                error!("VM: Failed to create file access");
                return None;
            }

            debug!("VM: Creating text output");
            let das_tout = das_text_make_printer();
            if das_tout.is_null() {
                error!("VM: Failed to create text output");
                das_fileaccess_release(das_fs);
                return None;
            }

            debug!("VM: Creating module group");
            let das_libs = das_modulegroup_make();
            if das_libs.is_null() {
                error!("VM: Failed to create module group");
                das_fileaccess_release(das_fs);
                das_text_release(das_tout);
                return None;
            }

            let state = VMState {
                hanged: false,
                tracked: Vec::new(),
            };

            Some(Self {
                das_fs,
                das_tout,
                das_libs,
                state: Arc::new(RwLock::new(state)),
                sys_progs: HashMap::new(),
            })
        }
    }

    pub fn load(&mut self, path: &str) -> Option<Arc<VMProgram>> {
        if let Some(prog) = VMProgram::new(
            self.state.clone(),
            path,
            self.das_fs,
            self.das_tout,
            self.das_libs,
        ) {
            self.sys_progs.insert(path.to_string(), Arc::new(prog));
            self.sys_progs.get(path).map(|f| f.clone())
        } else {
            None
        }
    }
}

#[cfg(feature = "free")]
impl Drop for VMEngine {
    fn drop(&mut self) {
        let mut sref = self.state.write();
        if !sref.hanged {
            sref.hanged = true;
            unsafe {
                debug!("VM: DANGERously VMEngine is memdropped");

                das_fileaccess_release(self.das_fs);
                das_text_release(self.das_tout);
                das_modulegroup_release(self.das_libs);

                for ctx in &mut sref.tracked {
                    ctx.release();
                }

                self.sys_progs.clear();

                das_shutdown();
            }
        } else {
            debug!("VM: VMEngine is flushed twice?");
        }
    }
}

/// The system to load a program and compile it, prepared for context hosting
pub struct VMProgram {
    state: Arc<RwLock<VMState>>,
    program: *mut das_program,
}

impl VMProgram {
    /// Creates a new DaScriptExecutable from the given script path.
    fn new(
        state: Arc<RwLock<VMState>>,
        script_path: &str,
        das_fs: *mut das_file_access,
        das_tout: *mut das_text_writer,
        das_libs: *mut das_module_group,
    ) -> Option<Self> {
        let c_script_path = match CString::new(script_path) {
            Ok(s) => s,
            Err(_) => {
                error!("Invalid string path");
                return None;
            }
        };

        unsafe {
            debug!("VM: Compiling program: {}", script_path);
            let program = das_program_compile(
                c_script_path.as_ptr().cast_mut(),
                das_fs,
                das_tout,
                das_libs,
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
                        das_error_output(error, das_tout);
                    }
                }
            }

            // do not release global object
            // debug!("VM: Finished compilation, releasing");
            // das_fileaccess_release(das_fs);
            // das_modulegroup_release(das_libs);
            // das_text_release(das_tout);

            if program.is_null() {
                error!("VM: Failed to compile program");
                None
            } else {
                Some(VMProgram { state, program })
            }
        }
    }

    /// Hosts the compiled program and returns a VMContext.
    pub fn host(&self) -> Option<VMHangedLock<VMContext>> {
        VMContext::new(self.state.clone(), self.program)
    }
}

#[cfg(feature = "free")]
impl Drop for VMProgram {
    fn drop(&mut self) {
        unsafe {
            debug!("VM: Releasing program");
            das_program_release(self.program);
        }
    }
}

/// A locked context to sync with VMState
pub struct VMHangedLock<T: VMHang>(pub Arc<RwLock<VMHanging<T>>>);

impl<T: VMHang> VMHangedLock<T> {
    fn new(inner: T) -> Self {
        Self(Arc::new(RwLock::new(VMHanging {
            destroyed: false,
            hanged: Some(Box::new(inner)),
        })))
    }

    fn release(&mut self) {
        let mut wref = self.0.write();
        if !wref.destroyed {
            wref.destroyed = true;
            wref.hanged = None;
        }
    }
}

impl<T: VMHang> Clone for VMHangedLock<T> {
    fn clone(&self) -> Self {
        VMHangedLock::<T>(Arc::clone(&&self.0))
    }
}

/// A context hosted from `VMProgram`, manage the context
pub struct VMContext {
    context: *mut das_context,
    // tout: *mut das_text_writer,
}

impl VMContext {
    /// Creates a new VMContext
    fn new(state: Arc<RwLock<VMState>>, program: *mut das_program) -> Option<VMHangedLock<Self>> {
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
                das_text_release(tout);
                None
            } else {
                das_text_release(tout);
                let hanging = VMHangedLock::new(VMContext {
                    context,
                    // tout
                });
                state.write().tracked.push(hanging.clone());
                Some(hanging)
            }
        }
    }
}

impl VMHangedLock<VMContext> {
    /// Find and evaluate a function by name
    pub fn eval_function(&self, name: &str) -> bool {
        debug!("VM: Evaluating function '{}'", name);

        // weird lifetime and scope hacking just to get the rawptr
        let lockref = self.0.read();
        let vmctx = match &lockref.hanged {
            Some(inner) => inner,
            None => panic!("unwrap failed"),
        };


        unsafe {
            let c_name = match CString::new(name) {
                Ok(s) => s,
                Err(_) => {
                    error!("Invalid function name");
                    return false;
                }
            };
            debug!("VM: Finding function pointer");
            let function = das_context_find_function(vmctx.context, c_name.as_ptr().cast_mut());
            if function.is_null() {
                error!("Function '{}' not found", name);
                return false;
            }

            // debug!("EXT: Validate function pointer");
            // if !dasx_verif_fn(function, c_name.into_raw()) {
            //     error!("Pointer is unsanitized");
            //     return false;
            // }

            debug!("VM: Evaluating function with catch");
            // let mut nullptr_allocated = [0f32, 0f32, 0f32, 0f32];

            let mut args = V4FloatUnlined::default();
            let mut ret = V4FloatUnlined::default();

            das_context_eval_with_catch_unaligned(
                vmctx.context,
                function,
                args.raw(),
                0,
                ret.raw(),
            );
            let exception = das_context_get_exception(vmctx.context);
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

#[cfg(feature = "free")]
impl Drop for VMContext {
    fn drop(&mut self) {
        unsafe {
            debug!("VM: Releasing context ctx");
            das_context_release(self.context);
            // debug!("VM: Releasing context tout");
            // das_text_release(self.tout);
        }
    }
}

impl VMHang for VMContext {}

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

// #[no_mangle]
// /// Load program into context
// pub extern "C" fn engine_load_program(
//     path: *const std::os::raw::c_char,
//     len: usize,
// ) -> *mut VMProgram {
//     if path.is_null() {
//         error!("Null path provided");
//         return std::ptr::null_mut();
//     }

//     debug!("Loading program from raw path");
//     let script_path = unsafe {
//         match std::str::from_utf8(std::slice::from_raw_parts(path as *const u8, len)) {
//             Ok(s) => s,
//             Err(_) => {
//                 error!("Invalid UTF-8 in path");
//                 return std::ptr::null_mut();
//             }
//         }
//     };

//     let executable = VMProgram::new(script_path);
//     match executable {
//         Some(exe) => Box::into_raw(Box::new(exe)),
//         None => std::ptr::null_mut(),
//     }
// }
