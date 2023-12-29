use winit::{
    error::{EventLoopError, OsError},
    event::{Event, WindowEvent},
    event_loop::{EventLoop, EventLoopBuilder},
    window::{Window, WindowBuilder},
};

use wgpu::util::DeviceExt;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version)]
pub struct Options {
    #[arg(long, default_value_t = 512)]
    pub width: u32,

    #[arg(long, default_value_t = 512)]
    pub height: u32,

    #[arg(long)]
    pub record: bool,

    #[arg(long)]
    pub verbose: bool,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Uniform {
    pub time: f32,

    _padding: u32,

    pub resolution: [f32; 2],
}

impl Uniform {
    fn new(width: u32, height: u32) -> Self {
        Self {
            time: 0.,

            _padding: 0,

            resolution: [width as f32, height as f32],
        }
    }
}

pub trait WindowState {
    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>);
    fn render(&mut self) -> Result<(), wgpu::SurfaceError>;
}

pub trait RecordState {
    fn record(&mut self, i: i32);
}

pub fn create_event_loop() -> Result<EventLoop<()>, EventLoopError> {
    let event_loop = EventLoopBuilder::new().build()?;

    event_loop.listen_device_events(winit::event_loop::DeviceEvents::Never);
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);

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

pub async fn create_adapter(
    instance: &wgpu::Instance,
    compatible_surface: Option<&wgpu::Surface>,
) -> wgpu::Adapter {
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

pub fn create_uniforms(
    device: &wgpu::Device,
    width: u32,
    height: u32,
) -> (
    Uniform,
    wgpu::Buffer,
    wgpu::BindGroupLayout,
    wgpu::BindGroup,
) {
    let uniform = Uniform::new(width, height);

    let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Uniforms"),
        contents: bytemuck::cast_slice(&[uniform]),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    let uniform_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("Uniform Bind Group Layout"),
        });

    let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &uniform_bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: uniform_buffer.as_entire_binding(),
        }],
        label: Some("Uniform Bind Group"),
    });

    return (
        uniform,
        uniform_buffer,
        uniform_bind_group_layout,
        uniform_bind_group,
    );
}

pub fn create_render_pipeline(
    device: &wgpu::Device,
    render_pipeline_layout: wgpu::PipelineLayout,
    vert_shader: wgpu::ShaderModule,
    frag_shader: wgpu::ShaderModule,
    vertex_buffers: &[wgpu::VertexBufferLayout<'_>],
    headless: bool,
) -> wgpu::RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(&render_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &vert_shader,
            entry_point: "vs_main",
            buffers: vertex_buffers,
        },
        fragment: Some(wgpu::FragmentState {
            module: &frag_shader,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: if headless {
                    wgpu::TextureFormat::Rgba8UnormSrgb
                } else {
                    wgpu::TextureFormat::Bgra8Unorm
                },
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

pub fn create_render_pass<'a>(
    encoder: &'a mut wgpu::CommandEncoder,
    view: &'a wgpu::TextureView,
) -> wgpu::RenderPass<'a> {
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
        present_mode: wgpu::PresentMode::Mailbox,
        alpha_mode: wgpu::CompositeAlphaMode::Auto,
        view_formats: vec![],
    }
}

pub fn create_texture_desc(texture_size: u32) -> wgpu::TextureDescriptor<'static> {
    wgpu::TextureDescriptor {
        size: wgpu::Extent3d {
            width: texture_size,
            height: texture_size,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        view_formats: &[],
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::RENDER_ATTACHMENT,
        label: None,
    }
}

pub fn copy_texture_to_buffer(
    encoder: &mut wgpu::CommandEncoder,
    texture: &wgpu::Texture,
    output_buffer: &wgpu::Buffer,
    texture_size: u32,
) {
    encoder.copy_texture_to_buffer(
        wgpu::ImageCopyTexture {
            aspect: wgpu::TextureAspect::All,
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
        },
        wgpu::ImageCopyBuffer {
            buffer: output_buffer,
            layout: wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * texture_size),
                rows_per_image: Some(texture_size),
            },
        },
        wgpu::Extent3d {
            width: texture_size,
            height: texture_size,
            depth_or_array_layers: 1,
        },
    );
}

pub fn create_output_buffer_desc(texture_size: u32) -> wgpu::BufferDescriptor<'static> {
    let output_buffer_size = (4 * texture_size * texture_size) as wgpu::BufferAddress;
    wgpu::BufferDescriptor {
        size: output_buffer_size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        label: None,
        mapped_at_creation: false,
    }
}

pub async fn save_buffer_as_image(
    output_buffer: &wgpu::Buffer,
    device: &wgpu::Device,
    texture_size: u32,
    name: &str,
) {
    let buffer_slice = output_buffer.slice(..);

    let (sender, receiver) = flume::bounded(1);

    buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
        sender.send(result).unwrap();
    });

    device.poll(wgpu::Maintain::Wait);
    receiver.recv_async().await.unwrap().unwrap();

    let data = buffer_slice.get_mapped_range();

    use image::{ImageBuffer, Rgba};
    let buffer = ImageBuffer::<Rgba<u8>, _>::from_raw(texture_size, texture_size, data).unwrap();
    buffer.save(format!("images/{}.png", name)).unwrap();
}

pub fn render(
    event_loop: EventLoop<()>,
    window: Window,
    mut state: impl WindowState,
) -> Result<(), EventLoopError> {
    let state_window_id = window.id();

    event_loop.run(move |event, elwt| match event {
        Event::WindowEvent { event, window_id } if window_id == state_window_id => match event {
            WindowEvent::CloseRequested => elwt.exit(),
            WindowEvent::Resized(physical_size) => state.resize(physical_size),
            WindowEvent::RedrawRequested => match state.render() {
                Ok(_) => {}
                _ => elwt.exit(),
            },
            _ => (),
        },

        Event::AboutToWait => window.request_redraw(),
        _ => (),
    })
}

pub fn record(mut state: impl RecordState, frames: i32) {
    for i in 0..frames {
        state.record(i);
    }
}
