#![no_main]

use libfuzzer_sys::fuzz_target;
use wirt::DummyErr;
use wirt::Wirt;
use wirt::WirtWarn;

#[derive(Debug, Clone)]
pub struct MyWarn(pub String);

impl core::fmt::Display for MyWarn {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "warn: {}", self.0)
    }
}

impl WirtWarn for MyWarn {
    fn clone_box(&self) -> Box<dyn WirtWarn> {
        Box::new(self.clone())
    }
}

fuzz_target!(|data: &[u8]| {
    if data.len() < 2 {
        return;
    }

    let mut w: Wirt<i32> = match data[0] % 3 {
        0 => Wirt::Ok(data.len() as i32),
        1 => Wirt::Err(&DummyErr {}, Some("fuzz error")),
        _ => Wirt::default(),
    };

    if data[1] > 128 {
        for i in 0..(data[1] as usize % 5) {
            w.add_warn(Box::new(MyWarn(format!("warn-{}", i))));
        }
    }

    if data[0] % 5 == 0 {
        let cloned_w = w.clone();
        let _ = core::hint::black_box(cloned_w);
    }

    match data.get(2).map(|&b| b % 4) {
        Some(0) => {
            let _ = core::hint::black_box(w.ok());
        }
        Some(1) => {
            let _ = core::hint::black_box(w.unwrap_err());
        }
        Some(2) => {
            let _ = w.ok();
            w.send_log();
        }
        _ => {}
    }
});
