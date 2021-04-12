#[allow(unused)]
#[allow(non_camel_case_types)]
mod hangul;

#[link(name = "hangul", kind = "dylib")]
extern "C" {
}

pub use hangul::{
    hangul_ic_delete, hangul_ic_flush, hangul_ic_get_commit_string, hangul_ic_get_preedit_string,
    hangul_ic_new, hangul_ic_process, hangul_ic_reset, HangulInputContext,
};

pub use kime_engine_backend::*;
pub use kime_engine_backend_hangul::*;
