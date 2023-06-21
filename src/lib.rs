#![feature(lazy_cell)]
#![feature(allocator_api)]
use std::{alloc::AllocError, fmt::Debug, rc::Rc};

pub mod buffer_id;
pub mod cpu;
pub mod device;
pub mod dtype;
pub mod shape;
pub mod storage;
pub mod tensor;
pub mod webgpu;

pub use buffer_id::*;
pub use cpu::*;
pub use device::*;
pub use dtype::*;
pub use shape::*;
pub use storage::*;
pub use tensor::*;
pub use webgpu::*;

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn it_works() {
        let t = Tensor::<CPU>::new(vec![2, 2].into(), vec![1., 2., 3., 4.]).unwrap();
        println!("{:#?}", t);
        let wgpu_device = WebGPU::new().await.unwrap();
        let gpu_t = t.to(wgpu_device);
        println!("{:#?}", gpu_t);
    }
}
