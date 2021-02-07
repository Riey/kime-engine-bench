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

struct TestKey {
    code: KeyCode,
    preedit: char,
}

impl TestKey {
    pub const fn new(code: KeyCode, preedit: char) -> Self {
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

    for TestKey { code, preedit } in set.keys.iter() {
        let ch = code.to_char();
        let retval = hangul_ic_process(hic, ch as u32 as _);

        let preedit_s = hangul_ic_get_preedit_string(hic);

        if cfg!(features = "check") {
            if preedit_s.is_null() {
                assert_eq!(*preedit, '\0');
            } else {
                assert_eq!(*preedit as u32, preedit_s.read())
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
        let mut ret_preedit = '\0';

        match ret.ty {
            InputResultType::ToggleHangul => {}
            InputResultType::ClearPreedit => {}
            InputResultType::Bypass => {
                commit.push(code.to_char());
            }
            InputResultType::Commit => {
                commit.push(ret.char1);
            }
            InputResultType::CommitBypass => {
                commit.push(ret.char1);
                commit.push(code.to_char());
            }
            InputResultType::Preedit => {
                ret_preedit = ret.char1;
            }
            InputResultType::CommitPreedit => {
                commit.push(ret.char1);
                ret_preedit = ret.char2;
            }
            InputResultType::CommitCommit => {
                commit.push(ret.char1);
                commit.push(ret.char2);
            }
        }

        if cfg!(feature = "check") {
            assert_eq!(ret_preedit, *preedit);
        }
    }

    let reset = engine.reset();
    if reset != '\0' {
        commit.push(reset);
    }

    if cfg!(feature = "check") {
        assert_eq!(commit, set.commit);
    }
}

fn libhangul(c: &mut Criterion) {
    let set = TestSet {
        keys: vec![
            TestKey::new(KeyCode::A, 'ㅁ'),
            TestKey::new(KeyCode::K, '마'),
            TestKey::new(KeyCode::S, '만'),
        ],
        commit: "만".into(),
    };

    c.bench_function("libhangul_keycode_commit", |b| {
        b.iter(|| unsafe {
            let hic = hangul_ic_new(cs!("2"));
            test_libhangul(hic, &set);
            hangul_ic_delete(hic);
        });
    });
}

fn kime_engine(c: &mut Criterion) {
    let set = TestSet {
        keys: vec![
            TestKey::new(KeyCode::A, 'ㅁ'),
            TestKey::new(KeyCode::K, '마'),
            TestKey::new(KeyCode::S, '만'),
        ],
        commit: "만".into(),
    };

    let config = Config::default();

    c.bench_function("kime_engine_keycode_commit", |b| {
        b.iter(|| {
            let mut engine = InputEngine::new();
            test_kime_engine(&mut engine, &config, &set);
        });
    });
}

criterion_group!(keyinput, libhangul, kime_engine);
criterion_main!(keyinput);
