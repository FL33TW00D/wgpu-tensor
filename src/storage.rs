use crate::device::Device;
use crate::{AllocMode, CPUPrim, TData, CPU};
use std::alloc::Layout;
use std::fmt::Debug;
use std::mem::ManuallyDrop;
use std::rc::Rc;

#[derive(thiserror::Error, Debug)]
pub enum StorageError {
    #[error("Failed to send data to device")]
    SendError(#[from] crate::DeviceError),
    #[error("Invalid layout requested: {0}")]
    InvalidLayout(#[from] std::alloc::LayoutError),
    #[error("Attempted to access empty storage")]
    EmptyStorage,
}

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
    ///Copy storage from the current device to an external device.
    ///Similar to Pytorch's [`to`](https://pytorch.org/docs/stable/generated/torch.Tensor.to.html) method.
    pub fn to<Ext: Device>(&self, ext: Ext) -> Result<Storage<Ext>, StorageError> {
        let mut dst = ext.allocate(self.layout, AllocMode::COPY_SRC | AllocMode::COPY_DST)?;
        self.device.copy_to(&self.data, &mut dst, &ext)?;

        Ok(Storage {
            data: dst,
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

    pub fn device(&self) -> &Rc<D> {
        &self.device
    }
}

impl Storage<CPU> {
    pub fn new<T: TData>(content: Vec<T>) -> Result<Self, StorageError> {
        let dt = T::dtype();
        let layout = Layout::from_size_align(content.len() * dt.size_of(), dt.alignment())?;

        let mut content = ManuallyDrop::new(content);
        let ptr = content.as_mut_ptr() as *mut u8;

        Ok(Storage {
            data: CPUPrim::new(ptr, layout.size()),
            layout,
            device: Rc::new(CPU),
        })
    }

    pub fn as_ptr<T: TData>(&self) -> Result<*const T, StorageError> {
        let ptr: *const T = self.data.as_ptr();
        if ptr.is_null() {
            Err(StorageError::EmptyStorage)
        } else {
            Ok(ptr)
        }
    }
}

impl<D: Device> Drop for Storage<D> {
    fn drop(&mut self) {
        self.device.deallocate(&mut self.data, self.layout).unwrap();
    }
}
