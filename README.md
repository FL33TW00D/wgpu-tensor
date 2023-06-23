# wgpu-tensor

```rust
    #[tokio::test]
    async fn it_works() {
        let data: Vec<f32> = vec![1., 2., 3., 4., 5., 6., 7., 8.];
        let original = Tensor::<CPU>::new(vec![2, 4].into(), data.clone()).unwrap();

        let wgpu_device = WebGPU::new().await.unwrap();

        let gpu_tensor = original.to(wgpu_device).unwrap();
        let returned = gpu_tensor.to(CPU).unwrap();
        assert_eq!(returned.as_slice::<f32>().unwrap(), data.as_slice());
    }
```

Exploring what a frictionless Tensor API would look like for CPU + GPU interop.

Based off [TensorGraph](https://github.com/conradludgate/tensorgraph) by Conrad Ludgate 
