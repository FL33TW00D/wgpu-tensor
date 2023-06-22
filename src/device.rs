use std::alloc::{AllocError, Layout};
use std::fmt::Debug;
use std::mem::MaybeUninit;

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

///Device is an abstraction for a device on which memory can be allocated.
///Devices only work on bytes, storage handles higher level types.
pub trait Device {
    ///The allocator used to allocate memory on the device.
    ///* CPU: std::alloc::System
    ///* WEBGPU: wgpu::Device
    type Allocator: DeviceAllocator + ?Sized;
    ///The primitive type used to represent memory on the device.
    ///* CPU: [`CPUPrim`]
    ///* WEBGPU: [`wgpu::Buffer`]
    type Prim: DevicePrimitive;
    fn copy_from_host(&self, src: &[u8], dst: &mut Self::Prim) -> Result<(), AllocError>;
    fn copy_to_host(&self, src: &Self::Prim, dst: &mut [u8]) -> Result<(), AllocError>;
    fn copy_to<Ext: Device>(
        &self,
        src: &Self::Prim,
        dst: &mut Ext::Prim,
        ext: &Ext,
    ) -> Result<(), AllocError> {
        //Default implementation does a roundtrip through the host.
        let mut buf: Vec<MaybeUninit<u8>> = Vec::with_capacity(src.len());
        unsafe {
            buf.set_len(src.len());
        }
        let buf_slice =
            unsafe { std::slice::from_raw_parts_mut(buf.as_mut_ptr() as *mut u8, src.len()) };
        self.copy_to_host(src, buf_slice)?;
        ext.copy_from_host(buf_slice, dst)?;
        Ok(())
    }
    fn allocate(&self, layout: Layout, mode: AllocMode) -> Result<Self::Prim, AllocError>;
    fn deallocate(&self, item: &mut Self::Prim, layout: Layout) -> Result<(), AllocError>;
}

///DeviceAllocator is similar to [`std::alloc::GlobalAlloc`], but allows different allocation modes.
pub trait DeviceAllocator {
    ///The primitive type used to represent memory on the device.
    ///* CPU: CPUPrim
    ///* WEBGPU: wgpu::Buffer
    type Prim;
    ///Allocates memory on the device.
    ///# Safety
    ///* The memory must be properly aligned.
    unsafe fn alloc(&self, layout: Layout, mode: AllocMode) -> Self::Prim;
    ///Allocates memory on the device and initializes it with the given data.
    ///# Safety
    ///* The memory must be properly aligned.
    ///* The data must be of the correct length.
    unsafe fn alloc_init(&self, layout: Layout, init: &[u8], mode: AllocMode) -> Self::Prim;
    ///Deallocates memory on the device.
    ///# Safety
    ///* The memory must be properly aligned.
    unsafe fn dealloc(&self, item: &mut Self::Prim, layout: Layout);
}

///Marker trait allowing for runtime type checking of device primitives.
pub trait DevicePrimitive: Debug {
    ///Returns the size of the primitive in bytes.
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
