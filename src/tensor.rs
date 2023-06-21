use std::{alloc::AllocError, rc::Rc};

use crate::{DType, Device, Shape, Storage, Strides, TData, CPU};

///Tensor  is a generalization of vectors and matrices to potentially higher dimensions.
///Internally, it uses [`Storage`] to store the data.
///This decouples the tensor from the underlying memory, so in essence, every tensor is a view into a storage.
///T: The type of the elements in the tensor.
///D: The device on which the tensor is stored.
#[derive(Debug)]
pub struct Tensor<D: Device> {
    dt: DType,
    shape: Shape,
    strides: Strides,
    storage: Rc<Storage<D>>,
}

impl<D: Device> Tensor<D> {
    pub fn to<Other: Device>(self, device: Other) -> Result<Tensor<Other>, anyhow::Error> {
        let storage = self.storage.to(device)?;
        Ok(Tensor {
            dt: self.dt,
            shape: self.shape,
            strides: self.strides,
            storage: Rc::new(storage),
        })
    }
}

impl Tensor<CPU> {
    ///Instantiates a new tensor on the CPU.
    ///You cannot instantiate a tensor on any other device, you can move the tensor
    ///from CPU -> D using [`Tensor::to`].
    pub fn new<T: TData>(shape: Shape, data: Vec<T>) -> Result<Self, AllocError> {
        let dt = T::dtype();
        let strides = shape.clone().into();
        let storage = Storage::new(data)?;

        Ok(Self {
            dt,
            shape,
            strides,
            storage: storage.into(),
        })
    }
}
