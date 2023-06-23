use crate::{AllocMode, Device, DeviceAllocator, DeviceError, DevicePrimitive};

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

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl DeviceAllocator for CPU {
    type Prim = CPUPrim;

    unsafe fn alloc(&self, layout: std::alloc::Layout, _mode: AllocMode) -> Self::Prim {
        Self::Prim {
            ptr: unsafe { std::alloc::alloc(layout) },
            len: layout.size(),
        }
    }

    unsafe fn alloc_init(
        &self,
        layout: std::alloc::Layout,
        init: &[u8],
        _mode: AllocMode,
    ) -> Self::Prim {
        if layout.size() != init.len() {
            panic!("Layout size does not match init size");
        }
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

    fn copy_from_host(&self, src: &[u8], dst: &mut Self::Prim) -> Result<(), DeviceError> {
        if src.len() != dst.len() {
            return Err(DeviceError::CopyMismatch(src.len(), dst.len()));
        }
        unsafe { std::ptr::copy_nonoverlapping(src.as_ptr(), dst.ptr, src.len()) };
        Ok(())
    }

    fn copy_to_host(&self, src: &Self::Prim, dst: &mut [u8]) -> Result<(), DeviceError> {
        if src.len() != dst.len() {
            return Err(DeviceError::CopyMismatch(src.len(), dst.len()));
        }
        unsafe { std::ptr::copy_nonoverlapping(src.ptr, dst.as_mut_ptr(), src.len()) };
        Ok(())
    }

    fn allocate(
        &self,
        layout: std::alloc::Layout,
        mode: AllocMode,
    ) -> Result<Self::Prim, DeviceError> {
        unsafe { Ok(Self::Allocator::alloc(self, layout, mode)) }
    }

    fn deallocate(
        &self,
        item: &mut Self::Prim,
        layout: std::alloc::Layout,
    ) -> Result<(), DeviceError> {
        unsafe { Self::Allocator::dealloc(self, item, layout) };
        Ok(())
    }
}
