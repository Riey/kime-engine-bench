#[allow(unused)]
#[allow(non_camel_case_types)]
mod hangul;

pub use hangul::{
    hangul_ic_delete, hangul_ic_flush, hangul_ic_get_commit_string, hangul_ic_get_preedit_string,
    hangul_ic_new, hangul_ic_process, hangul_ic_reset, HangulInputContext,
};

pub use kime_engine_cffi::{Config, InputCategory, InputEngine, InputResult, InputResult_CONSUMED, InputResult_HAS_PREEDIT, InputResult_LANGUAGE_CHANGED, InputResult_HAS_COMMIT};
