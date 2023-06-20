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
    ALL,
}

///Device is an abstraction for a device on which memory can be allocated.
pub trait Device {
    ///The primitive type used to represent memory on the device.
    ///* CPU: *mut u8
    ///* WEBGPU: wgpu::Buffer
    type Prim: Debug;
    fn copy_to_host<T: TData>(&self, src: Self::Prim, dst: &mut [T]);
    fn copy_from_host<T: TData>(&self, src: &[T], dst: Self::Prim);
}
