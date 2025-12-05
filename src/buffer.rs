pub trait Buffer {
    fn init_buffer(&mut self, device: &wgpu::Device);
    fn bind_group(&self) -> Option<&wgpu::BindGroup>;
    fn bind_group_layout(&self) -> Option<&wgpu::BindGroupLayout>;
    fn write_buffer(&self, queue: &wgpu::Queue);
}
