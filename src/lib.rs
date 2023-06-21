#![feature(lazy_cell)]
#![feature(allocator_api)]
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
        let data: Vec<f32> = vec![1., 2., 3., 4., 5., 6., 7., 8.];
        let t = Tensor::<CPU>::new(vec![2, 4].into(), data).unwrap();
        println!("Original CPU Tensor: {}", t);
        let wgpu_device = WebGPU::new().await.unwrap();
        let gpu_t = t.to(wgpu_device).unwrap();
        println!("GPU Tensor: {:#?}", gpu_t);
        let returned = gpu_t.to(CPU).unwrap();
        println!("Final CPU Tensor: {}", returned);
    }
}
