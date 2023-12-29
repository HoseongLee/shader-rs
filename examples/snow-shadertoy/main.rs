use clap::Parser;
use simple_logger::SimpleLogger;

const TEXTURE_SIZE: u32 = 1024u32;

struct WindowState {
    device: wgpu::Device,
    queue: wgpu::Queue,

    surface: wgpu::Surface,
    config: wgpu::SurfaceConfiguration,

    render_pipeline: wgpu::RenderPipeline,

    uniform: shader_rs::Uniform,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,

    start: std::time::Instant,
}

struct RecordState {
    device: wgpu::Device,
    queue: wgpu::Queue,

    texture: wgpu::Texture,
    texture_view: wgpu::TextureView,
    output_buffer: wgpu::Buffer,

    render_pipeline: wgpu::RenderPipeline,

    uniform: shader_rs::Uniform,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
}

impl WindowState {
    async fn new(
        window: &winit::window::Window,
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

        let frag_shader = device.create_shader_module(frag_shader_desc);
        let vert_shader = device.create_shader_module(vert_shader_desc);

        let (uniform, uniform_buffer, uniform_bind_group_layout, uniform_bind_group) =
            shader_rs::create_uniforms(&device, size.width, size.height);

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
            &[],
            false,
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

impl shader_rs::WindowState for WindowState {
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
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

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

impl RecordState {
    async fn new(
        frag_shader_desc: wgpu::ShaderModuleDescriptor<'_>,
        vert_shader_desc: wgpu::ShaderModuleDescriptor<'_>,
    ) -> Self {
        let instance = shader_rs::create_instance();

        let adapter = shader_rs::create_adapter(&instance, None).await;

        let (device, queue) = shader_rs::create_device_and_queue(&adapter).await;

        let texture_desc = shader_rs::create_texture_desc(TEXTURE_SIZE);

        let texture = device.create_texture(&texture_desc);
        let texture_view = texture.create_view(&Default::default());

        let output_buffer_desc = shader_rs::create_output_buffer_desc(TEXTURE_SIZE);

        let output_buffer = device.create_buffer(&output_buffer_desc);

        let frag_shader = device.create_shader_module(frag_shader_desc);
        let vert_shader = device.create_shader_module(vert_shader_desc);

        let (uniform, uniform_buffer, uniform_bind_group_layout, uniform_bind_group) =
            shader_rs::create_uniforms(&device, TEXTURE_SIZE, TEXTURE_SIZE);

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
            &[],
            true,
        );

        Self {
            device,
            queue,

            texture,
            texture_view,
            output_buffer,

            render_pipeline,

            uniform,
            uniform_buffer,
            uniform_bind_group,
        }
    }

    fn update(&mut self, i: f32) {
        self.uniform.time = i / 30.0;

        self.queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&[self.uniform]),
        );
    }
}

impl shader_rs::RecordState for RecordState {
    fn record(&mut self, i: i32) {
        self.update(i as f32);

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        {
            let mut render_pass = shader_rs::create_render_pass(&mut encoder, &self.texture_view);

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
            render_pass.draw(0..3, 0..1);
        }

        shader_rs::copy_texture_to_buffer(
            &mut encoder,
            &self.texture,
            &self.output_buffer,
            TEXTURE_SIZE,
        );

        self.queue.submit(Some(encoder.finish()));

        {
            pollster::block_on(shader_rs::save_buffer_as_image(
                &self.output_buffer,
                &self.device,
                TEXTURE_SIZE,
                &format!("{:0>8}", i),
            ));
        }

        self.output_buffer.unmap();
    }
}

fn main() {
    let args = shader_rs::Options::parse();

    if args.verbose {
        SimpleLogger::new().init().unwrap();
    }

    let frag_shader_desc = wgpu::include_wgsl!("shader.wgsl");
    let vert_shader_desc = wgpu::include_wgsl!("vertex.wgsl");

    if args.record {
        let state = pollster::block_on(RecordState::new(frag_shader_desc, vert_shader_desc));
        let _ = shader_rs::record(state, 300);
    } else {
        let event_loop = shader_rs::create_event_loop().unwrap();
        let window = shader_rs::create_window(args.width, args.height, &event_loop).unwrap();

        let state = pollster::block_on(WindowState::new(
            &window,
            frag_shader_desc,
            vert_shader_desc,
        ));
        let _ = shader_rs::render(event_loop, window, state);
    }
}
