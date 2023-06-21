use std::alloc::AllocError;

use crate::{AllocMode, Device, DeviceAllocator, DevicePrimitive, TData};

impl DeviceAllocator for CPU {
    type Prim = *mut u8;

    unsafe fn alloc(&self, layout: std::alloc::Layout, mode: AllocMode) -> Self::Prim {
        unsafe { std::alloc::alloc(layout) }
    }

    unsafe fn alloc_init(
        &self,
        layout: std::alloc::Layout,
        init: &[u8],
        mode: AllocMode,
    ) -> Self::Prim {
        let ptr = unsafe { std::alloc::alloc(layout) };
        unsafe { std::ptr::copy_nonoverlapping(init.as_ptr(), ptr, init.len()) };
        ptr
    }

    unsafe fn dealloc(&self, item: &mut Self::Prim, layout: std::alloc::Layout) {
        unsafe { std::alloc::dealloc(*item, layout) }
    }
}

impl DevicePrimitive for *mut u8 {
    fn as_bytes(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(*self, 1) }
    }

    fn as_bytes_mut(&mut self) -> &mut [u8] {}
}

///Default device
#[derive(Debug)]
pub struct CPU;

impl Device for CPU {
    type Prim = *mut u8;
    type Allocator = CPU;

    fn copy_from_host<T: TData>(&self, src: &[T], dst: &Self::Prim) -> Result<(), AllocError> {
        //Copying from Host to Host is a no-op
        Err(AllocError)
    }

    fn copy_to_host<T: TData>(&self, src: &Self::Prim, dst: &mut [T]) -> Result<(), AllocError> {
        //Copying from Host to Host is a no-op
        Err(AllocError)
    }

    fn copy_to<Ext: Device>(
        &self,
        src: &Ext::Prim,
        dst: &mut Self::Prim,
        len: usize,
    ) -> Result<(), AllocError> {
        todo!();
    }

    fn allocate(
        &self,
        layout: std::alloc::Layout,
        mode: AllocMode,
    ) -> Result<Self::Prim, AllocError> {
        unsafe { Ok(Self::Allocator::alloc(self, layout, mode)) }
    }

    fn deallocate(
        &self,
        item: &mut Self::Prim,
        layout: std::alloc::Layout,
    ) -> Result<(), AllocError> {
        unsafe { Self::Allocator::dealloc(self, item, layout) };
        Ok(())
    }
}
