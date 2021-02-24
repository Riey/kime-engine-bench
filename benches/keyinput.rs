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
    preedit: &'static str,
}

impl TestKey {
    pub const fn new(code: KeyCode, preedit: &'static str) -> Self {
        Self { code, preedit }
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
    let mut preedit_buf = String::with_capacity(64);

    for TestKey { code, preedit } in set.keys.iter() {
        let ch = code.to_char();
        let retval = hangul_ic_process(hic, ch as u32 as _);

        if cfg!(features = "check") {
            let preedit_s = hangul_ic_get_preedit_string(hic);
            if preedit_s.is_null() {
                assert!(preedit.is_empty());
            } else {
                append_c_str(&mut preedit_buf, preedit_s);
                assert_eq!(*preedit, preedit_buf);
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
    let mut commit = String::with_capacity(set.commit.len());
    engine.set_hangul_enable(true);

    for TestKey { code, preedit } in set.keys.iter() {
        let ret = engine.press_key(config, code.to_keycode(), 0);

        if cfg!(feature = "check") {
            if ret & InputResult_HAS_PREEDIT != 0 {
                assert_eq!(*preedit, engine.preedit_str());
            } else {
                assert!(preedit.is_empty());
            }
        }

        if ret & (InputResult_NEED_FLUSH | InputResult_NEED_RESET) != 0 {
            commit.push_str(engine.commit_str());

            if ret & InputResult_NEED_RESET != 0 {
                engine.reset();
            } else {
                engine.flush();
            }
        }

        if ret & InputResult_CONSUMED == 0 {
            commit.push(code.to_char());
        }
    }

    engine.clear_preedit();
    commit.push_str(engine.commit_str());

    if cfg!(feature = "check") {
        assert_eq!(commit, set.commit);
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
        b.iter(|| unsafe {
            let hic = hangul_ic_new(cs!("2"));
            test_libhangul(hic, &set);
            hangul_ic_delete(hic);
        });
    });

    c.bench_function("libhangul_keycode_commit_50", |b| {
        let set = get_testset(50);
        b.iter(|| unsafe {
            let hic = hangul_ic_new(cs!("2"));
            test_libhangul(hic, &set);
            hangul_ic_delete(hic);
        });
    });

    c.bench_function("libhangul_keycode_commit_500", |b| {
        let set = get_testset(500);
        b.iter(|| unsafe {
            let hic = hangul_ic_new(cs!("2"));
            test_libhangul(hic, &set);
            hangul_ic_delete(hic);
        });
    });
}

fn kime_engine(c: &mut Criterion) {
    let config = Config::default();

    c.bench_function("kime_engine_keycode_commit_5", |b| {
        let set = get_testset(5);
        b.iter(|| {
            let mut engine = InputEngine::new(&config);
            test_kime_engine(&mut engine, &config, &set);
        });
    });

    c.bench_function("kime_engine_keycode_commit_50", |b| {
        let set = get_testset(50);
        b.iter(|| {
            let mut engine = InputEngine::new(&config);
            test_kime_engine(&mut engine, &config, &set);
        });
    });

    c.bench_function("kime_engine_keycode_commit_500", |b| {
        let set = get_testset(500);
        b.iter(|| {
            let mut engine = InputEngine::new(&config);
            test_kime_engine(&mut engine, &config, &set);
        });
    });
}

criterion_group!(keyinput, libhangul, kime_engine);
criterion_main!(keyinput);
