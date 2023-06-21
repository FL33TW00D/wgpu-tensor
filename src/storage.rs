use std::alloc::Layout;

use crate::device::Device;

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
    pub fn new() {
        todo!()
    }

    ///Create a new Storage on the given device.
    pub fn to<Ext: Device>(&self, device: Ext) -> Storage<Ext> {}
}
