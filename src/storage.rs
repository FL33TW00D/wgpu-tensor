use std::alloc::{AllocError, Layout};

use crate::device::Device;
use crate::{AllocMode, TData, CPU};

///Storage is an abstraction that allows us to decouple a Tensor from its data.
///It is a contiguous one-dimensional array of elements of a single data type.
///The data is allocated on a device, and can be "moved" between devices.
///* B: The handle to the underlying data.
///* D: The device on which the data is stored.
#[derive(Debug)]
pub struct Storage<D: Device> {
    data: D::Prim,
    layout: Layout,
    device: D,
}

impl<D: Device> Storage<D> {
    pub fn to<Other: Device>(self, device: Other) -> Storage<Other> {
        let 
        Storage {
            data,
            layout: self.layout,
            device,
        }
    }
}

impl Storage<CPU> {
    pub fn new<T: TData>(data: &[T]) -> Result<Self, AllocError> {
        let dt = T::dtype();
        let bytes = data.len() * dt.size_of();
        let layout = std::alloc::Layout::from_size_align(bytes, dt.alignment()).unwrap();

        let data = CPU.allocate_init(data, AllocMode::ALL)?;

        Ok(Self {
            data,
            layout,
            device: CPU,
        })
    }
}
