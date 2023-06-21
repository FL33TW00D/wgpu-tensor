use crate::device::Device;
use crate::{AllocMode, TData, CPU};
use std::alloc::{AllocError, Layout};
use std::fmt::Debug;
use std::mem::ManuallyDrop;
use std::rc::Rc;

///Storage is an abstraction that allows us to decouple a Tensor from its data.
///It is a contiguous one-dimensional array of elements of a single data type.
///The data is allocated on a device, and can be "moved" between devices.
///* D: The device on which the data is stored.
#[derive(Debug)]
pub struct Storage<D: Device> {
    data: D::Prim,
    layout: Layout,
    device: Rc<D>,
}

impl<D: Device> Storage<D> {
    ///Create a new Storage on the given device.
    pub fn to<Ext: Device>(&self, ext: Ext) -> Result<Storage<Ext>, anyhow::Error> {
        let prim = ext.allocate(self.layout, AllocMode::DEFAULT)?;
        //Here we should cast to bytes
        self.device
            .copy_to(&self.data, &prim, self.layout.size(), &ext)?;
        Ok(Storage {
            data: prim,
            layout: self.layout,
            device: Rc::new(ext),
        })
    }

    pub fn data(&self) -> &D::Prim {
        &self.data
    }

    pub fn layout(&self) -> &Layout {
        &self.layout
    }
}

impl Storage<CPU> {
    pub fn new<T: TData>(content: Vec<T>) -> Result<Self, AllocError> {
        let dt = T::dtype();
        let layout = Layout::from_size_align(content.len() * dt.size_of(), dt.alignment()).unwrap();

        let mut content = ManuallyDrop::new(content);
        let data = content.as_mut_ptr() as *mut u8;

        Ok(Storage {
            data,
            layout,
            device: Rc::new(CPU),
        })
    }
}

impl<D: Device> Drop for Storage<D> {
    fn drop(&mut self) {
        let _ = self.device.deallocate(&mut self.data, self.layout);
    }
}
