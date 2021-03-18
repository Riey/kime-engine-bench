use criterion::{criterion_group, criterion_main, Criterion};
use kime_engine_bench::*;

macro_rules! cs {
    ($e:expr) => {
        concat!($e, "\0").as_ptr().cast()
    };
}

#[derive(Clone, Copy)]
#[repr(u16)]
enum KeyCode {
    A = 38,
    K = 45,
    S = 39,
}

impl KeyCode {
    pub const fn to_char(self) -> char {
        match self {
            KeyCode::A => 'a',
            KeyCode::K => 'k',
            KeyCode::S => 's',
        }
    }

    pub const fn to_keycode(self) -> u16 {
        self as u16
    }
}

#[derive(Clone, Copy)]
struct TestKey {
    code: KeyCode,
    #[cfg(features = "check")]
    preedit: &'static str,
}

impl TestKey {
    #[allow(unused_variables)]
    pub const fn new(code: KeyCode, preedit: &'static str) -> Self {
        #[cfg(features = "check")]
        {
            Self { code, preedit }
        }
        #[cfg(not(features = "check"))]
        {
            Self { code }
        }
    }
}

struct TestSet {
    keys: Vec<TestKey>,
    commit: String,
}

unsafe fn append_c_str(out: &mut String, mut s: *const u32) {
    if s.is_null() {
        return;
    }

    let mut c = s.read();

    while c != 0 {
        out.push(std::char::from_u32_unchecked(c));
        s = s.add(1);
        c = s.read();
    }
}

unsafe fn test_libhangul(hic: *mut HangulInputContext, set: &TestSet) {
    let mut commit = String::with_capacity(set.commit.len());
    #[cfg(features = "check")]
    let mut preedit_buf = String::with_capacity(64);

    for key in set.keys.iter() {
        let ch = key.code.to_char();
        let retval = hangul_ic_process(hic, ch as u32 as _);

        #[cfg(features = "check")]
        {
            let preedit_s = hangul_ic_get_preedit_string(hic);
            if preedit_s.is_null() {
                assert!(key.preedit.is_empty());
            } else {
                append_c_str(&mut preedit_buf, preedit_s);
                assert_eq!(*key.preedit, preedit_buf);
            }
        }

        if !retval {
            let s = hangul_ic_flush(hic);
            append_c_str(&mut commit, s);
        }
    }

    append_c_str(&mut commit, hangul_ic_flush(hic));

    if cfg!(features = "check") {
        assert_eq!(commit, set.commit);
    }
}

fn test_kime_engine(engine: &mut InputEngine, config: &Config, set: &TestSet) {
    engine.set_input_category(InputCategory::Hangul);

    for key in set.keys.iter() {
        #[allow(unused_variables)]
        let ret = engine.press_key(config, key.code.to_keycode(), 0);

        #[cfg(features = "check")]
        {
            if ret & InputResult_HAS_PREEDIT != 0 {
                assert_eq!(*preedit, engine.preedit_str());
            } else {
                assert!(preedit.is_empty());
            }

            assert_ne!(ret & InputResult_CONSUMED, 0);
        }
    }

    engine.clear_preedit();

    if cfg!(feature = "check") {
        assert_eq!(engine.commit_str(), set.commit);
    }
}

fn get_testset(count: usize) -> TestSet {
    TestSet {
        keys: [
            TestKey::new(KeyCode::A, "ㅁ"),
            TestKey::new(KeyCode::K, "마"),
            TestKey::new(KeyCode::S, "만"),
        ]
        .repeat(count),
        commit: "만".repeat(count),
    }
}

fn libhangul(c: &mut Criterion) {
    c.bench_function("libhangul_keycode_commit_5", |b| {
        let set = get_testset(5);
        let hic = unsafe { hangul_ic_new(cs!("2")) };
        b.iter(|| unsafe {
            test_libhangul(hic, &set);
            hangul_ic_reset(hic);
        });
    });

    c.bench_function("libhangul_keycode_commit_50", |b| {
        let set = get_testset(50);
        let hic = unsafe { hangul_ic_new(cs!("2")) };
        b.iter(|| unsafe {
            test_libhangul(hic, &set);
            hangul_ic_reset(hic);
        });
    });

    c.bench_function("libhangul_keycode_commit_500", |b| {
        let set = get_testset(500);
        let hic = unsafe { hangul_ic_new(cs!("2")) };
        b.iter(|| unsafe {
            test_libhangul(hic, &set);
            hangul_ic_reset(hic);
        });
    });
}

fn kime_engine(c: &mut Criterion) {
    let config = Config::default();

    c.bench_function("kime_engine_keycode_commit_5", |b| {
        let set = get_testset(5);
        let mut engine = InputEngine::new(&config);
        b.iter(|| {
            test_kime_engine(&mut engine, &config, &set);
            engine.reset();
        });
    });

    c.bench_function("kime_engine_keycode_commit_50", |b| {
        let set = get_testset(50);
        let mut engine = InputEngine::new(&config);
        b.iter(|| {
            test_kime_engine(&mut engine, &config, &set);
            engine.reset();
        });
    });

    c.bench_function("kime_engine_keycode_commit_500", |b| {
        let set = get_testset(500);
        let mut engine = InputEngine::new(&config);
        b.iter(|| {
            test_kime_engine(&mut engine, &config, &set);
            engine.reset();
        });
    });
}

criterion_group!(keyinput, libhangul, kime_engine);
criterion_main!(keyinput);
