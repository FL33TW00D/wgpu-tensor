use std::alloc::AllocError;

use crate::{AllocMode, Device, DeviceAllocator, TData};

#[derive(Debug)]
pub struct CPUAllocator;

impl DeviceAllocator for CPUAllocator {
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

    unsafe fn dealloc(&self, item: Self::Prim, layout: std::alloc::Layout) {
        unsafe { std::alloc::dealloc(item, layout) }
    }
}

///Default device
#[derive(Debug)]
pub struct CPU {
    allocator: CPUAllocator,
}

impl Device for CPU {
    type Prim = *mut u8;
    type Allocator = CPUAllocator;

    fn copy_from_host<T: TData>(&self, src: &[T], dst: Self::Prim) -> Result<(), AllocError> {
        //Here we don't actually need to copy anything, since the data is already on the CPU.
        //We just need to make sure that the pointer is valid.
        //
        todo!()
    }

    fn copy_to_host<T: TData>(&self, src: Self::Prim, dst: &mut [T]) -> Result<(), AllocError> {
        todo!()
    }
}
