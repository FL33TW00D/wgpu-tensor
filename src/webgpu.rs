use bytemuck::NoUninit;
use std::alloc::AllocError;
use std::alloc::Layout;
use wgpu::util::DeviceExt;
use wgpu::util::DownloadBuffer;

use wgpu::InstanceDescriptor;
use wgpu::Limits;

use crate::AllocMode;
use crate::DType;
use crate::Device;
use crate::Shape;
use crate::TData;

#[derive(Debug)]
pub struct GPUHandle {
    device: wgpu::Device, //Responsible for the creation of compute resources.
    queue: wgpu::Queue,   //Executes recorded CommandBuffers.
}

impl GPUHandle {
    // Get a device and a queue, honoring WGPU_ADAPTER_NAME and WGPU_BACKEND environment variables
    pub async fn new() -> Result<Self, anyhow::Error> {
        let backends = wgpu::util::backend_bits_from_env().unwrap_or(wgpu::Backends::PRIMARY);
        let instance = wgpu::Instance::new(InstanceDescriptor {
            backends,
            ..Default::default()
        });
        let adapter = wgpu::util::initialize_adapter_from_env_or_default(&instance, backends, None)
            .await
            .expect("No GPU found given preference");

        // `request_device` instantiates the feature specific connection to the GPU, defining some parameters,
        //  `features` being the available features.
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("rumble"),
                    features: wgpu::Features::default(),
                    limits: Limits::default(),
                },
                None,
            )
            .await
            .expect("Could not create adapter for GPU device");

        Ok(Self { device, queue })
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }
}

///Thin wrapper over our GPUHandle for issuing commands to the GPU.
#[derive(Debug)]
pub struct WebGPU {
    handle: GPUHandle,
}

impl WebGPU {
    pub async fn new() -> Result<Self, anyhow::Error> {
        Ok(Self {
            handle: GPUHandle::new().await?,
        })
    }
}

impl Device for WebGPU {
    type Prim = wgpu::Buffer;

    fn copy_to_host<T: TData>(&self, src: Self::Prim, dst: &mut [T]) {
        let buffer_slice = src.slice(..);
        let (tx, rx) = std::sync::mpsc::sync_channel(1);

        wgpu::util::DownloadBuffer::read_buffer(
            self.handle.device(),
            self.handle.queue(),
            &buffer_slice,
            move |buffer| {
                // Called on download completed
                tx.send(if let Ok(b) = buffer {
                    unsafe {
                        let bytes = std::slice::from_raw_parts(b.as_ptr() as *const u8, b.len());
                        let dt = T::dtype();
                        let size = dt.size_of() * dst.len();
                        let mut dst =
                            std::slice::from_raw_parts_mut(dst.as_mut_ptr() as *mut u8, size);
                        dst.copy_from_slice(bytes);
                    }
                } else {
                    panic!("Failed to download buffer")
                })
                .unwrap();
            },
        );
        self.handle.device().poll(wgpu::Maintain::Wait);
        rx.recv().unwrap()
    }

    fn copy_from_host<T: TData>(&self, src: &[T], dst: Self::Prim) {}
}
