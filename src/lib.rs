#![feature(allocator_api)]
#![feature(lazy_cell)]
use smallvec::SmallVec;
use std::{alloc::AllocError, fmt::Debug, rc::Rc};

pub mod buffer_id;
pub mod cpu;
pub mod device;
pub mod dtype;
pub mod storage;
pub mod webgpu;

pub use buffer_id::*;
pub use cpu::*;
pub use device::*;
pub use dtype::*;
pub use storage::*;
pub use webgpu::*;

#[derive(Debug, Clone)]
pub struct Shape(SmallVec<[usize; 4]>);

impl Shape {
    pub fn new<D>(d: D) -> Self
    where
        D: Into<SmallVec<[usize; 4]>>,
    {
        Self(d.into())
    }

    pub fn iter(&self) -> impl Iterator<Item = &usize> {
        self.0.iter()
    }

    pub fn numel(&self) -> usize {
        self.0.iter().product()
    }
}

impl From<Vec<usize>> for Shape {
    fn from(v: Vec<usize>) -> Self {
        Self(v.into())
    }
}

#[derive(Debug, Clone)]
pub struct Strides(SmallVec<[usize; 4]>);

impl From<Shape> for Strides {
    fn from(shape: Shape) -> Self {
        let mut strides = SmallVec::with_capacity(shape.0.len());
        let mut stride = 1;
        for dim in shape.0.iter().rev() {
            strides.push(stride);
            stride *= dim;
        }
        Self(strides)
    }
}

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
    pub fn to<Other: Device>(self, device: Other) -> Tensor<Other> {
        let storage = self.storage.to(device);
        Tensor {
            dt: self.dt,
            shape: self.shape,
            strides: self.strides,
            storage: Rc::new(storage),
        }
    }
}

impl Tensor<CPU> {
    pub fn new<T: TData>(shape: Shape, data: &[T]) -> Result<Self, AllocError> {
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

impl Into<wgpu::BufferUsages> for AllocMode {
    fn into(self) -> wgpu::BufferUsages {
        match self {
            AllocMode::MAP_READ => wgpu::BufferUsages::MAP_READ,
            AllocMode::MAP_WRITE => wgpu::BufferUsages::MAP_WRITE,
            AllocMode::COPY_READ => wgpu::BufferUsages::COPY_SRC,
            AllocMode::COPY_WRITE => wgpu::BufferUsages::COPY_DST,
            AllocMode::STORAGE => wgpu::BufferUsages::STORAGE,
            AllocMode::ALL => wgpu::BufferUsages::all(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn it_works() {
        let t = Tensor::<CPU>::new(vec![2, 2].into(), &[1., 2., 3., 4.]).unwrap();
        println!("{:?}", t);
        let wgpu_device = WebGPU::new().await.unwrap();
        let gpu_t = t.to(wgpu_device);
        println!("{:?}", gpu_t);
    }
}
