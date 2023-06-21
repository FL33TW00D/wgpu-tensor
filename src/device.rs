use std::alloc::{AllocError, Layout};
use std::fmt::Debug;

use crate::TData;

///When we allocate memory, we need to specify how we want to use it.
#[allow(non_camel_case_types)]
pub enum AllocMode {
    MAP_READ,
    MAP_WRITE,
    COPY_READ,
    COPY_WRITE,
    STORAGE,
    DEFAULT,
}

impl Into<wgpu::BufferUsages> for AllocMode {
    fn into(self) -> wgpu::BufferUsages {
        match self {
            AllocMode::MAP_READ => wgpu::BufferUsages::MAP_READ,
            AllocMode::MAP_WRITE => wgpu::BufferUsages::MAP_WRITE,
            AllocMode::COPY_READ => wgpu::BufferUsages::COPY_SRC,
            AllocMode::COPY_WRITE => wgpu::BufferUsages::COPY_DST,
            AllocMode::STORAGE => wgpu::BufferUsages::STORAGE,
            AllocMode::DEFAULT => wgpu::BufferUsages::all(),
        }
    }
}

///Device is an abstraction for a device on which memory can be allocated.
pub trait Device {
    ///The allocator used to allocate memory on the device.
    ///* CPU: std::alloc::System
    ///* WEBGPU: wgpu::Device
    type Allocator: DeviceAllocator;
    ///The primitive type used to represent memory on the device.
    ///* CPU: *mut u8
    ///* WEBGPU: wgpu::Buffer
    type Prim: Debug;
    fn copy_from_host<T: TData>(&self, src: &[T], dst: Self::Prim) -> Result<(), AllocError>;
    fn copy_to_host<T: TData>(&self, src: Self::Prim, dst: &mut [T]) -> Result<(), AllocError>;
    fn copy_to<T: TData, Ext: Device>(
        &self,
        src: Self::Prim,
        dst: Ext::Prim,
        len: usize,
        dst_device: &Ext,
    ) -> Result<(), AllocError>;
    fn allocate(&self, layout: Layout, mode: AllocMode) -> Result<Self::Prim, AllocError>;
}

///DeviceAllocator is similar to [`std::alloc::GlobalAlloc`], but allows different allocation modes.
pub trait DeviceAllocator {
    ///The primitive type used to represent memory on the device.
    ///* CPU: *mut u8
    ///* WEBGPU: wgpu::Buffer
    type Prim;
    unsafe fn alloc(&self, layout: Layout, mode: AllocMode) -> Self::Prim;
    unsafe fn alloc_init(&self, layout: Layout, init: &[u8], mode: AllocMode) -> Self::Prim;
    unsafe fn dealloc(&self, item: Self::Prim, layout: Layout);
}
