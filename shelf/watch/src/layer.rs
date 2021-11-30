use std::sync::{Arc, Mutex};

pub enum Layer {
    Pipeline(
        wgpu::RenderPipeline,
        Box<dyn Fn(&crate::frame::Frame) -> bool>,
    ),
    Handle(
        Box<
            dyn FnMut(
                Arc<Mutex<crate::frame::Frame>>,
                &wgpu::TextureView,
                &mut wgpu::CommandEncoder,
            ),
        >,
    ),
}
