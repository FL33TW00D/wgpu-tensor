use std::alloc::AllocError;

use crate::{AllocMode, Device, DeviceAllocator, TData};

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
        src: &Self::Prim,
        dst: &Ext::Prim,
        len: usize,
        dst_device: &Ext,
    ) -> Result<(), AllocError> {
        dst_device.copy_from_host(
            unsafe { std::slice::from_raw_parts(*src as *const u8, len) },
            dst,
        )
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
