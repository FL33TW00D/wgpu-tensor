use std::alloc::AllocError;
use std::any::Any;
use std::ops::Range;

use crate::DevicePrimitive;
use crate::{BufferID, Device, DeviceAllocator, TData};
use wgpu::util::DeviceExt;
use wgpu::InstanceDescriptor;
use wgpu::Limits;

///Encapsulates everything needed to interact with the GPU.
#[derive(Debug)]
pub struct GPUHandle {
    device: wgpu::Device, //Responsible for the creation of compute resources.
    queue: wgpu::Queue,   //Executes recorded CommandBuffers.
}

impl GPUHandle {
    pub async fn new() -> Result<Self, anyhow::Error> {
        let backends = wgpu::util::backend_bits_from_env().unwrap_or(wgpu::Backends::PRIMARY);
        let instance = wgpu::Instance::new(InstanceDescriptor {
            backends,
            ..Default::default()
        });
        let adapter = wgpu::util::initialize_adapter_from_env_or_default(&instance, backends, None)
            .await
            .expect("No GPU found given preference");

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("rumble"),
                    features: wgpu::Features::default(),
                    limits: Limits::default(),
                },
                None,
            )
            .await?;

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
        self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(BufferID::new().inner()),
            size: layout.size() as u64,
            usage: mode.into(),
            mapped_at_creation: false,
        })
    }

    unsafe fn alloc_init(
        &self,
        _layout: std::alloc::Layout,
        init: &[u8],
        mode: crate::AllocMode,
    ) -> Self::Prim {
        self.device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(BufferID::new().inner()),
                contents: init,
                usage: mode.into(),
            })
    }

    unsafe fn dealloc(&self, item: &mut Self::Prim, _layout: std::alloc::Layout) {
        item.destroy()
    }
}

impl DevicePrimitive for wgpu::Buffer {
    fn write_bytes<WebGPU>(&self, device: &WebGPU, data: &[u8], range: Range<usize>) {
        device.queue().write_buffer(self, range.start as u64, data)
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

    pub fn handle(&self) -> &GPUHandle {
        &self.handle
    }
}

impl Device for WebGPU {
    type Prim = wgpu::Buffer;
    type Allocator = GPUHandle;

    fn copy_to_host<T: TData>(&self, src: &Self::Prim, dst: &mut [T]) -> Result<(), AllocError> {
        let buffer_slice = src.slice(..);
        let (tx, rx) = std::sync::mpsc::sync_channel(1);

        wgpu::util::DownloadBuffer::read_buffer(
            self.handle.device(),
            self.handle.queue(),
            &buffer_slice,
            move |buffer| {
                tx.send(if let Ok(b) = buffer {
                    let dt = T::dtype();
                    unsafe {
                        std::slice::from_raw_parts(b.as_ptr() as *const T, dt.size_of() * dst.len())
                    }
                } else {
                    panic!("Failed to download buffer")
                })
                .unwrap();
            },
        );
        self.handle.device().poll(wgpu::Maintain::Wait);
        let result = rx.recv().unwrap();
        dst.copy_from_slice(result);
        Ok(())
    }

    fn copy_from_host<T: TData>(&self, src: &[T], dst: &Self::Prim) -> Result<(), AllocError> {
        Ok(())
    }

    fn copy_to<Ext: Device>(
        &self,
        src: &Self::Prim,
        range: Range<usize>,
        ext: &Ext,
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

    fn deallocate(
        &self,
        item: &mut Self::Prim,
        layout: std::alloc::Layout,
    ) -> Result<(), AllocError> {
        unsafe { self.handle.dealloc(item, layout) }
        Ok(())
    }
}
