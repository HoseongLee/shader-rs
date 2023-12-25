use winit::{
    error::{EventLoopError, OsError},
    event::{Event, WindowEvent},
    event_loop::{EventLoop, EventLoopBuilder},
    window::{Window, WindowId, WindowBuilder},
};

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version)]
pub struct Options {
    #[arg(long, default_value_t = 512)]
    pub width: u32,

    #[arg(long, default_value_t = 512)]
    pub height: u32,

    #[arg(long)]
    pub verbose: bool,
}

pub trait WindowState {
    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>);
    fn render(&mut self) -> Result<(), wgpu::SurfaceError>;
}

pub fn create_event_loop() -> Result<EventLoop<()>, EventLoopError> {
    let event_loop = EventLoopBuilder::new().build()?;

    event_loop.listen_device_events(winit::event_loop::DeviceEvents::Never);
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

    Ok(event_loop)
}

pub fn create_window(
    width: u32,
    height: u32,
    event_loop: &EventLoop<()>,
) -> Result<Window, OsError> {
    WindowBuilder::new()
        .with_title("Shader-rs")
        .with_inner_size(winit::dpi::PhysicalSize::new(width, height))
        .build(event_loop)
}

pub fn create_instance() -> wgpu::Instance {
    wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::VULKAN,
        ..Default::default()
    })
}

pub async fn create_adapter(instance: &wgpu::Instance, compatible_surface: Option<&wgpu::Surface>) -> wgpu::Adapter {
    instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface,
            force_fallback_adapter: false,
        })
        .await
        .unwrap()
}

pub async fn create_device_and_queue(adapter: &wgpu::Adapter) -> (wgpu::Device, wgpu::Queue) {
    adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
            },
            None,
        )
        .await
        .unwrap()
}

pub fn create_render_pipeline(
    device: &wgpu::Device,
    render_pipeline_layout: wgpu::PipelineLayout,
    vert_shader: wgpu::ShaderModule,
    frag_shader: wgpu::ShaderModule,
) -> wgpu::RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(&render_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &vert_shader,
            entry_point: "vs_main",
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: &frag_shader,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: wgpu::TextureFormat::Bgra8Unorm,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
    })
}

pub fn create_render_pass<'a>(encoder: &'a mut wgpu::CommandEncoder, view: &'a wgpu::TextureView) -> wgpu::RenderPass<'a> {
    encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Render Pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 1.0,
                }),
                store: wgpu::StoreOp::Store,
            },
        })],

        depth_stencil_attachment: None,
        occlusion_query_set: None,
        timestamp_writes: None,
    })
}

pub fn surface_config(width: u32, height: u32) -> wgpu::SurfaceConfiguration {
    wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: wgpu::TextureFormat::Bgra8Unorm,
        width,
        height,
        present_mode: wgpu::PresentMode::Fifo,
        alpha_mode: wgpu::CompositeAlphaMode::Auto,
        view_formats: vec![],
    }
}

pub fn render(
    event_loop: EventLoop<()>,
    state_window_id: WindowId,
    mut state: impl WindowState,
) -> Result<(), EventLoopError> {
    event_loop.run(move |event, elwt| match event {
        Event::WindowEvent { event, window_id } if window_id == state_window_id => match event {
            WindowEvent::CloseRequested => elwt.exit(),
            WindowEvent::Resized(physical_size) => state.resize(physical_size),
            _ => (),
        },

        Event::AboutToWait => match state.render() {
            Ok(_) => {}
            _ => elwt.exit(),
        },

        _ => (),
    })
}
