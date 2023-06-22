use std::{alloc::AllocError, ops::Range};

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
    fn write_bytes<CPU>(&self, _device: &CPU, data: &[u8], range: Range<usize>) {
        unsafe { std::ptr::copy_nonoverlapping(data.as_ptr(), *self, range.len()) };
    }
}

///Default device
#[derive(Debug)]
pub struct CPU;

impl Device for CPU {
    type Prim = *mut u8;
    type Allocator = CPU;

    fn copy_from_host<T: TData>(&self, src: &[T], dst: &mut Self::Prim) -> Result<(), AllocError> {
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
        range: Range<usize>,
        ext: &Ext,
    ) -> Result<Ext::Prim, AllocError> {
        let layout = std::alloc::Layout::from_size_align(range.len(), 1).unwrap();
        let dst = ext.allocate(layout, AllocMode::DEFAULT)?;
        let src_slice = unsafe { std::slice::from_raw_parts(*src, range.len()) };
        dst.write_bytes(ext, src_slice, range);
        Ok(dst)
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
