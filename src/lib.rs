#![feature(allocator_api)]
pub mod alloc_mode;
pub mod buffer_id;
pub mod cpu;
pub mod device;
pub mod dtype;
pub mod shape;
pub mod storage;
pub mod tensor;
pub mod webgpu;

pub use alloc_mode::*;
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
        let data: Vec<f32> = vec![1., 2., 3., 4., 5., 6., 7., 8.];
        let original = Tensor::<CPU>::new(vec![2, 4].into(), data.clone()).unwrap();

        let wgpu_device = WebGPU::new().await.unwrap();

        let gpu_tensor = original.to(wgpu_device).unwrap();
        let returned = gpu_tensor.to(CPU).unwrap();
        assert_eq!(returned.as_slice::<f32>().unwrap(), data.as_slice());
    }
}
