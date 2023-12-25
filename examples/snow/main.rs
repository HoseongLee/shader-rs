use clap::Parser;
use simple_logger::SimpleLogger;

use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniform {
    time: f32,

    _padding: u32,

    resolution: [f32; 2],
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

struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    render_pipeline: wgpu::RenderPipeline,

    uniform: Uniform,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,

    start: std::time::Instant,

    _window: winit::window::Window,
}

impl State {
    async fn new(
        window: winit::window::Window,
        frag_shader_desc: wgpu::ShaderModuleDescriptor<'_>,
        vert_shader_desc: wgpu::ShaderModuleDescriptor<'_>,
    ) -> Self {
        let size = window.inner_size();

        let instance = shader_rs::create_instance();

        let surface = unsafe { instance.create_surface(&window) }.unwrap();

        let adapter = shader_rs::create_adapter(&instance, Some(&surface)).await;

        let (device, queue) = shader_rs::create_device_and_queue(&adapter).await;

        let config = shader_rs::surface_config(size.width, size.height);

        surface.configure(&device, &config);

        let uniform = Uniform::new(size.width, size.height);

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

        let frag_shader = device.create_shader_module(frag_shader_desc);
        let vert_shader = device.create_shader_module(vert_shader_desc);

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&uniform_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = shader_rs::create_render_pipeline(
            &device,
            render_pipeline_layout,
            vert_shader,
            frag_shader,
        );

        Self {
            surface,
            device,
            queue,
            config,
            render_pipeline,

            uniform,
            uniform_buffer,
            uniform_bind_group,

            start: std::time::Instant::now(),
            _window: window,
        }
    }

    fn update(&mut self) {
        self.uniform.time = self.start.elapsed().as_secs_f32();

        self.queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&[self.uniform]),
        );
    }
}

impl shader_rs::WindowState for State {
    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.uniform.resolution = [new_size.width as f32, new_size.height as f32];
            self.surface.configure(&self.device, &self.config);
        }
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.update();
        
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        let mut render_pass = shader_rs::create_render_pass(&mut encoder, &view);

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);

        render_pass.draw(0..3, 0..1);

        drop(render_pass);

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

fn main() {
    let args = shader_rs::Options::parse();

    if args.verbose {
        SimpleLogger::new().init().unwrap();
    }

    let frag_shader_desc = wgpu::include_wgsl!("shader.wgsl");
    let vert_shader_desc = wgpu::include_wgsl!("vertex.wgsl");

    let event_loop = shader_rs::create_event_loop().unwrap();
    let window = shader_rs::create_window(args.width, args.height, &event_loop).unwrap();

    let window_id = window.id();

    let state = pollster::block_on(State::new(window, frag_shader_desc, vert_shader_desc));

    let _ = shader_rs::render(event_loop, window_id, state);
}
