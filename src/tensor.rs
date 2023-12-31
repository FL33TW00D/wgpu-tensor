use std::rc::Rc;

use crate::{as_std, DType, Device, Shape, Storage, StorageError, Strides, TData, CPU};
use itertools::Itertools;

#[derive(thiserror::Error, Debug)]
pub enum TensorError {
    #[error("Provided shape: {0:?} does not match the # of elements: {1}")]
    ShapeMismatch(Shape, usize),
    #[error("Storage error: {0}")]
    StorageError(#[from] StorageError),
}

///Tensor is a generalization of vectors and matrices to potentially higher dimensions.
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

impl PartialEq for Tensor<CPU> {
    fn eq(&self, other: &Self) -> bool {
        if self.shape != other.shape {
            return false;
        }
        if self.dt != other.dt {
            return false;
        }
        unsafe fn eq_t<T: TData>(a: &Tensor<CPU>, b: &Tensor<CPU>) -> bool {
            a.as_slice::<T>().unwrap() == b.as_slice::<T>().unwrap()
        }
        unsafe { as_std!(eq_t(self.dt)(self, other)) }
    }
}

impl<D: Device> Tensor<D> {
    ///Moves the tensor from D -> Other.
    pub fn to<Ext: Device>(self, ext: Ext) -> Result<Tensor<Ext>, anyhow::Error> {
        let storage = self.storage.to(ext)?;
        Ok(Tensor {
            dt: self.dt,
            shape: self.shape,
            strides: self.strides,
            storage: Rc::new(storage),
        })
    }

    pub fn device(&self) -> &D {
        self.storage.device()
    }
}

impl Tensor<CPU> {
    ///Instantiates a new tensor on the CPU.
    ///You cannot instantiate a tensor on any other device
    ///To create a tensor on a Device, D, you can move the tensor
    ///from CPU -> D using [`Tensor::to`].
    pub fn new<T: TData>(shape: Shape, data: Vec<T>) -> Result<Self, TensorError> {
        if shape.numel() != data.len() {
            return Err(TensorError::ShapeMismatch(shape, data.len()));
        }
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

    pub fn as_slice<T: TData>(&self) -> anyhow::Result<&[T]> {
        let ptr: *const T = self.storage.as_ptr()?;
        if ptr.is_null() {
            Ok(&[])
        } else {
            unsafe { Ok(std::slice::from_raw_parts::<T>(ptr, self.shape.numel())) }
        }
    }
}

impl std::fmt::Display for Tensor<CPU> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unsafe fn dump_t<T: TData>(tensor: &Tensor<CPU>, n: usize) -> String {
            tensor.as_slice::<T>().unwrap()[0..n].iter().join(", ")
        }
        write!(f, "[{}]", unsafe {
            as_std!(dump_t(self.dt)(self, self.shape.numel()))
        })
    }
}
