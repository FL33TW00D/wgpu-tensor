use nanoid::{alphabet::SAFE, nanoid};
use rand::{prelude::StdRng, RngCore, SeedableRng};
use std::cell::LazyCell;
use std::sync::atomic::{AtomicU64, Ordering};

const IDCNT: LazyCell<AtomicU64> = LazyCell::new(|| AtomicU64::new(0));

pub struct BufferID(&'static str);

impl BufferID {
    const MAX_LENGTH: usize = 6;

    fn seed(size: usize) -> Vec<u8> {
        let mut rng: StdRng = SeedableRng::seed_from_u64(IDCNT.fetch_add(1, Ordering::SeqCst));
        let mut bytes: Vec<u8> = vec![0; size];
        rng.fill_bytes(&mut bytes);
        bytes
    }

    pub fn new() -> Self {
        Self(nanoid!(6, &SAFE, Self::seed).as_str())
    }
}
