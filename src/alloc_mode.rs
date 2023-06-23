//Wrapper around wgpu::BufferUsages
bitflags::bitflags! {
    #[repr(transparent)]
    pub struct AllocMode: u32 {
        const MAP_READ = 1 << 0;
        const MAP_WRITE = 1 << 1;
        const COPY_SRC = 1 << 2;
        const COPY_DST = 1 << 3;
        const UNIFORM = 1 << 6;
        const STORAGE = 1 << 7;
    }
}

impl From<AllocMode> for wgpu::BufferUsages {
    fn from(value: AllocMode) -> Self {
        let mut flags = Self::empty();
        if value.contains(AllocMode::MAP_READ) {
            flags |= Self::MAP_READ;
        }
        if value.contains(AllocMode::MAP_WRITE) {
            flags |= Self::MAP_WRITE;
        }
        if value.contains(AllocMode::COPY_SRC) {
            flags |= Self::COPY_SRC;
        }
        if value.contains(AllocMode::COPY_DST) {
            flags |= Self::COPY_DST;
        }
        if value.contains(AllocMode::UNIFORM) {
            flags |= Self::UNIFORM;
        }
        if value.contains(AllocMode::STORAGE) {
            flags |= Self::STORAGE;
        }
        flags
    }
}
