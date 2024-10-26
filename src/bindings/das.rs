//! C 2 CPP mapping
//!
//! https://github.com/GaijinEntertainment/daScript/blob/main/src/misc/daScriptC.cpp

#[path = "./c.rs"]
mod c;

#[allow(unused_imports)]
pub use c::{
    das_context, das_context_eval_with_catch, das_context_find_function, das_context_get_exception,
    das_context_make, das_context_release, das_enumeration, das_enumeration_add_value,
    das_enumeration_make, das_error, das_error_output, das_error_report, das_file_access,
    das_fileaccess_introduce_file, das_fileaccess_make_default, das_fileaccess_make_project,
    das_fileaccess_release, das_function, das_get_root, das_initialize, das_interop_function,
    das_module, das_module_bind_alias, das_module_bind_enumeration,
    das_module_bind_interop_function, das_module_bind_structure, das_module_create,
    das_module_group, das_modulegroup_add_module, das_modulegroup_make, das_modulegroup_release,
    das_node, das_program, das_program_compile, das_program_context_stack_size,
    das_program_err_count, das_program_get_error, das_program_release, das_program_simulate,
    das_shutdown, das_structure, das_structure_add_field, das_structure_make,
    das_text_make_printer, das_text_make_writer, das_text_output, das_text_release,
    das_text_writer,
};
