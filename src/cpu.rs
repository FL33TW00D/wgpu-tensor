use std::{alloc::AllocError, ops::Range};

use crate::{AllocMode, Device, DeviceAllocator, DevicePrimitive};

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
    type Device = CPU;
    fn write_bytes(&self, _device: &CPU, data: &[u8], range: Range<usize>) {
        unsafe { std::ptr::copy_nonoverlapping(data.as_ptr(), *self, range.len()) };
    }
}

///Default device
#[derive(Debug)]
pub struct CPU;

impl Device for CPU {
    type Prim = *mut u8;
    type Allocator = CPU;

    fn copy_from_host(&self, src: &[u8], dst: &mut Self::Prim) -> Result<(), AllocError> {
        unsafe { std::ptr::copy_nonoverlapping(src.as_ptr(), *dst, src.len()) };
        Ok(())
    }

    fn copy_to_host(&self, src: &Self::Prim, dst: &mut [u8]) -> Result<(), AllocError> {
        unsafe { std::ptr::copy_nonoverlapping(*src, dst.as_mut_ptr(), dst.len()) };
        Ok(())
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
