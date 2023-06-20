#![feature(allocator_api)]
use smallvec::SmallVec;
use std::{
    alloc::{AllocError, Allocator, Layout},
    fmt::Debug,
    rc::Rc,
};
use wgpu::util::DeviceExt;

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

#[derive(Debug, Clone, Default)]
pub enum DType {
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    F16,
    #[default]
    F32,
    F64,
}

macro_rules! dtype {
    ($t:ty, $v:ident) => {
        impl TData for $t {
            fn name() -> &'static str {
                stringify!($t)
            }

            fn dtype() -> DType {
                DType::$v
            }
        }
    };
}

dtype!(u8, U8);
dtype!(u16, U16);
dtype!(u32, U32);
dtype!(u64, U64);
dtype!(i8, I8);
dtype!(i16, I16);
dtype!(i32, I32);
dtype!(i64, I64);
dtype!(f32, F32);
dtype!(f64, F64);

#[macro_export]
macro_rules! as_std {
    ($($path:ident)::* ($dt:expr) ($($args:expr),*)) => { {
        match $dt {
          DType::U8   => $($path)::*::<u8>($($args),*),
          DType::U16  => $($path)::*::<u16>($($args),*),
          DType::U32  => $($path)::*::<u32>($($args),*),
          DType::U64  => $($path)::*::<u64>($($args),*),
          DType::I8   => $($path)::*::<i8>($($args),*),
          DType::I16  => $($path)::*::<i16>($($args),*),
          DType::I32  => $($path)::*::<i32>($($args),*),
          DType::I64  => $($path)::*::<i64>($($args),*),
          DType::F16  => $($path)::*::<i16>($($args),*),
          DType::F32  => $($path)::*::<f32>($($args),*),
          DType::F64  => $($path)::*::<f64>($($args),*),
        }
    } }
}

impl DType {
    #[inline]
    pub fn size_of(&self) -> usize {
        as_std!(std::mem::size_of(self)())
    }

    #[inline]
    pub fn alignment(&self) -> usize {
        self.size_of()
    }
}

///Marker trait for types that can be used as tensor data.
pub trait TData: bytemuck::Pod + bytemuck::Zeroable {
    fn name() -> &'static str;
    fn dtype() -> DType;
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

impl Tensor<CPU> {
    pub fn new<T: TData>(shape: Shape, data: &[T]) -> Result<Self, AllocError> {
        let dt = T::dtype();
        let strides = shape.clone().into();
        let bytes = shape.iter().cloned().product::<usize>() * dt.size_of();
        let layout = std::alloc::Layout::from_size_align(bytes, dt.alignment()).unwrap();

        let storage = Storage {
            data: CPU.allocate_init(data, AllocMode::ALL)?,
            layout,
            device: CPU,
        };

        Ok(Self {
            dt,
            shape,
            strides,
            storage: storage.into(),
        })
    }
}

///Storage is a contiguous, one-dimensional array of elements of a single data type.
///The data is allocated on a device, and can be moved between devices.
///* B: The handle to the underlying data.
///* D: The device on which the data is stored.
#[derive(Debug)]
pub struct Storage<D: Device> {
    data: D::Handle,
    layout: Layout,
    device: D,
}

impl Storage<CPU> {
    pub fn new<T: TData>(data: &[T]) -> Result<Self, AllocError> {
        let dt = T::dtype();
        let bytes = data.len() * dt.size_of();
        let layout = std::alloc::Layout::from_size_align(bytes, dt.alignment()).unwrap();

        let data = CPU.allocate(layout, AllocMode::STORAGE)?;

        Ok(Self {
            data,
            layout,
            device: CPU,
        })
    }
}

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

///Device is an abstraction for a device on which memory can be allocated.
pub trait Device {
    ///Handle is either:
    ///* A pointer to the allocated memory.
    ///* A buffer on the GPU.
    type Handle: Debug;
    fn allocate(&self, layout: Layout, mode: AllocMode) -> Result<Self::Handle, AllocError>;
    fn allocate_init<T: TData>(
        &self,
        contents: &[T],
        mode: AllocMode,
    ) -> Result<Self::Handle, AllocError>;
    fn deallocate(&self, data: Self::Handle, layout: Layout);
}

///Default device
#[derive(Debug)]
pub struct CPU;

pub struct WebGPU {
    _device: wgpu::Device,
}

impl Device for CPU {
    type Handle = *mut u8;
    fn allocate(&self, layout: Layout, _mode: AllocMode) -> Result<Self::Handle, AllocError> {
        Ok(unsafe { std::alloc::alloc(layout) })
    }

    fn deallocate(&self, data: Self::Handle, layout: Layout) {
        unsafe { std::alloc::dealloc(data, layout) }
    }

    fn allocate_init<T: TData>(
        &self,
        contents: &[T],
        mode: AllocMode,
    ) -> Result<Self::Handle, AllocError> {
        let layout = Layout::new::<T>();
        let data = self.allocate(layout, mode)?;
        unsafe {
            std::ptr::copy_nonoverlapping(contents.as_ptr(), data as *mut T, contents.len());
        }
        Ok(data)
    }
}

impl Device for WebGPU {
    type Handle = wgpu::Buffer;
    fn allocate(&self, layout: Layout, mode: AllocMode) -> Result<Self::Handle, AllocError> {
        Ok(self._device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: layout.size() as u64,
            usage: mode.into(),
            mapped_at_creation: false,
        }))
    }

    fn deallocate(&self, data: Self::Handle, _layout: Layout) {
        data.destroy()
    }

    fn allocate_init<T: TData>(
        &self,
        contents: &[T],
        mode: AllocMode,
    ) -> Result<Self::Handle, AllocError> {
        Ok(self
            ._device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(contents),
                usage: mode.into(),
            }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        let t = Tensor::<CPU>::new(vec![2, 2].into(), &[1., 2., 3., 4.]).unwrap();
        dbg!(t);
    }
}
