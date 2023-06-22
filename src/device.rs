use std::alloc::{AllocError, Layout};
use std::fmt::Debug;

///When we allocate memory, we need to specify how we want to use it.
#[allow(non_camel_case_types)]
pub enum AllocMode {
    MAP_READ,
    MAP_WRITE,
    COPY_READ,
    COPY_WRITE,
    STORAGE,
    ALL,
}

impl Into<wgpu::BufferUsages> for AllocMode {
    fn into(self) -> wgpu::BufferUsages {
        match self {
            AllocMode::MAP_READ => wgpu::BufferUsages::MAP_READ,
            AllocMode::MAP_WRITE => wgpu::BufferUsages::MAP_WRITE,
            AllocMode::COPY_READ => wgpu::BufferUsages::COPY_SRC,
            AllocMode::COPY_WRITE => wgpu::BufferUsages::COPY_DST,
            AllocMode::STORAGE => wgpu::BufferUsages::STORAGE,
            AllocMode::ALL => wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
        }
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
    ///* CPU: *mut u8
    ///* WEBGPU: wgpu::Buffer
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
        let mut buf = vec![0; src.len()];
        self.copy_to_host(src, &mut buf)?;
        ext.copy_from_host(&buf, dst)?;
        Ok(())
    }
    fn allocate(&self, layout: Layout, mode: AllocMode) -> Result<Self::Prim, AllocError>;
    fn deallocate(&self, item: &mut Self::Prim, layout: Layout) -> Result<(), AllocError>;
}

///DeviceAllocator is similar to [`std::alloc::GlobalAlloc`], but allows different allocation modes.
pub trait DeviceAllocator {
    ///The primitive type used to represent memory on the device.
    ///* CPU: *mut u8
    ///* WEBGPU: wgpu::Buffer
    type Prim;
    unsafe fn alloc(&self, layout: Layout, mode: AllocMode) -> Self::Prim;
    unsafe fn alloc_init(&self, layout: Layout, init: &[u8], mode: AllocMode) -> Self::Prim;
    unsafe fn dealloc(&self, item: &mut Self::Prim, layout: Layout);
}

///Marker trait allowing for runtime type checking of device primitives.
pub trait DevicePrimitive: Debug {
    ///Returns the size of the primitive in bytes.
    fn len(&self) -> usize;
}
