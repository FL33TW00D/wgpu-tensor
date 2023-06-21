use std::alloc::AllocError;

use crate::{BufferID, Device, DeviceAllocator, TData};
use wgpu::util::DeviceExt;
use wgpu::InstanceDescriptor;
use wgpu::Limits;

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

impl DeviceAllocator for GPUHandle {
    type Prim = wgpu::Buffer;

    unsafe fn alloc(&self, layout: std::alloc::Layout, mode: crate::AllocMode) -> Self::Prim {
        todo!()
    }

    unsafe fn alloc_init(
        &self,
        layout: std::alloc::Layout,
        init: &[u8],
        mode: crate::AllocMode,
    ) -> Self::Prim {
        todo!()
    }

    unsafe fn dealloc(&self, item: Self::Prim, layout: std::alloc::Layout) {
        todo!()
    }
}

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
    type Allocator = GPUHandle;

    fn copy_to_host<T: TData>(&self, src: Self::Prim, dst: &mut [T]) -> Result<(), AllocError> {
        let buffer_slice = src.slice(..);
        let (tx, rx) = std::sync::mpsc::sync_channel(1);

        wgpu::util::DownloadBuffer::read_buffer(
            self.handle.device(),
            self.handle.queue(),
            &buffer_slice,
            move |buffer| {
                tx.send(if let Ok(b) = buffer {
                    unsafe { std::slice::from_raw_parts(b.as_ptr() as *const u8, b.len()) }
                } else {
                    panic!("Failed to download buffer")
                })
                .unwrap();
            },
        );
        self.handle.device().poll(wgpu::Maintain::Wait);
        //TODO: bring inside closure
        let result = rx.recv().unwrap();
        let dt = T::dtype();
        let len = dt.size_of() * dst.len();
        let result = unsafe { std::slice::from_raw_parts(result.as_ptr() as *const T, len) };
        dst.copy_from_slice(result);
        Ok(())
    }

    fn copy_from_host<T: TData>(&self, src: &[T], dst: Self::Prim) -> Result<(), AllocError> {
        self.handle
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(BufferID::new().as_str()),
                contents: bytemuck::cast_slice(src),
                usage: wgpu::BufferUsages::COPY_SRC,
            });
        Ok(())
    }

    fn copy_to<T: TData, Ext: Device>(
        &self,
        src: Self::Prim,
        dst: Ext::Prim,
        len: usize,
        dst_device: &Ext,
    ) -> Result<(), AllocError> {
        todo!()
    }

    fn allocate(
        &self,
        layout: std::alloc::Layout,
        mode: crate::AllocMode,
    ) -> Result<Self::Prim, AllocError> {
        unsafe { Ok(self.handle.alloc(layout, mode)) }
    }
}
