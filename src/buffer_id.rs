use nanoid::{alphabet::SAFE, nanoid};
use once_cell::sync::Lazy;
use rand::{prelude::StdRng, RngCore, SeedableRng};
use std::sync::atomic::{AtomicU64, Ordering};

static IDCNT: Lazy<AtomicU64> = Lazy::new(|| AtomicU64::new(0));

#[derive(Default, Debug)]
pub struct BufferID(String);

impl BufferID {
    const MAX_LENGTH: usize = 6;

    fn seed(size: usize) -> Vec<u8> {
        let mut rng: StdRng = SeedableRng::seed_from_u64(IDCNT.fetch_add(1, Ordering::SeqCst));
        let mut bytes: Vec<u8> = vec![0; size];
        rng.fill_bytes(&mut bytes);
        bytes
    }

    pub fn new() -> Self {
        let length = Self::MAX_LENGTH;
        Self(nanoid!(length, &SAFE, Self::seed))
    }

    pub fn inner(&self) -> &str {
        &self.0
    }
}
