use crate::AllocMode;

use std::alloc::Layout;
use std::fmt::Debug;
use std::mem::MaybeUninit;

#[derive(thiserror::Error, Debug)]
pub enum DeviceError {
    #[error("Attempted to copy from source with length {0} to destination with length {1}")]
    CopyMismatch(usize, usize),
    #[error("Allocation error: {0}")]
    AllocError(#[from] std::alloc::AllocError),
    #[error("Error transferring data from device: {0} to host")]
    TransferError(String),
    #[error("Failed to obtain required resource: {0}")]
    ResourceError(#[from] anyhow::Error),
}

///Device is an abstraction for a device on which memory can be allocated.
///Devices only work on bytes, storage handles higher level types.
pub trait Device {
    ///The allocator used to allocate memory on the device.
    ///* CPU: [`std::alloc::System`]
    ///* WEBGPU: [`wgpu::Device`]
    type Allocator: DeviceAllocator + ?Sized;
    ///The primitive type used to represent memory on the device.
    ///* CPU: [`CPUPrim`]
    ///* WEBGPU: [`wgpu::Buffer`]
    type Prim: DevicePrimitive;
    fn copy_from_host(&self, src: &[u8], dst: &mut Self::Prim) -> Result<(), DeviceError>;
    fn copy_to_host(&self, src: &Self::Prim, dst: &mut [u8]) -> Result<(), DeviceError>;
    fn copy_to<Ext: Device>(
        &self,
        src: &Self::Prim,
        dst: &mut Ext::Prim,
        ext: &Ext,
    ) -> Result<(), DeviceError> {
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
    fn allocate(&self, layout: Layout, mode: AllocMode) -> Result<Self::Prim, DeviceError>;
    fn deallocate(&self, item: &mut Self::Prim, layout: Layout) -> Result<(), DeviceError>;
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
