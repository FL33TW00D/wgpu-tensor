use std::alloc::AllocError;

use crate::{AllocMode, Device, DeviceAllocator, DevicePrimitive};

///The CPU primitive for storing data.
///Much like a slice, but owned.
#[derive(Debug)]
pub struct CPUPrim {
    ptr: *mut u8,
    len: usize,
}
impl CPUPrim {
    pub fn new(ptr: *mut u8, len: usize) -> Self {
        Self { ptr, len }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn as_ptr<T>(&self) -> *const T {
        self.ptr as *const T
    }
}

impl DeviceAllocator for CPU {
    type Prim = CPUPrim;

    unsafe fn alloc(&self, layout: std::alloc::Layout, mode: AllocMode) -> Self::Prim {
        let ptr = unsafe { std::alloc::alloc(layout) };
        Self::Prim {
            ptr,
            len: layout.size(),
        }
    }

    unsafe fn alloc_init(
        &self,
        layout: std::alloc::Layout,
        init: &[u8],
        mode: AllocMode,
    ) -> Self::Prim {
        let ptr = unsafe { std::alloc::alloc(layout) };
        unsafe { std::ptr::copy_nonoverlapping(init.as_ptr(), ptr, init.len()) };
        Self::Prim {
            ptr,
            len: layout.size(),
        }
    }

    unsafe fn dealloc(&self, item: &mut Self::Prim, layout: std::alloc::Layout) {
        unsafe { std::alloc::dealloc(item.ptr, layout) };
    }
}

impl DevicePrimitive for CPUPrim {
    fn len(&self) -> usize {
        self.len()
    }
}

///Default device
#[derive(Debug)]
pub struct CPU;

impl Device for CPU {
    type Prim = CPUPrim;
    type Allocator = CPU;

    fn copy_from_host(&self, src: &[u8], dst: &mut Self::Prim) -> Result<(), AllocError> {
        unsafe { std::ptr::copy_nonoverlapping(src.as_ptr(), dst.ptr, src.len()) };
        Ok(())
    }

    fn copy_to_host(&self, src: &Self::Prim, dst: &mut [u8]) -> Result<(), AllocError> {
        unsafe { std::ptr::copy_nonoverlapping(src.ptr, dst.as_mut_ptr(), src.len()) };
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
