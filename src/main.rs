use clap::Parser;
use tokio::runtime::Runtime;
use simple_logger::SimpleLogger;

use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoopBuilder,
    window::WindowBuilder,
};

use shader_rs::State;


#[derive(Parser, Debug)]
#[command(version)]
struct Options {
    #[arg(short, long, default_value_t=512)]
    width: u32,

    #[arg(short, long, default_value_t=512)]
    height: u32,

    #[arg(short, long)]
    verbose: bool,
}

fn main() {
    let args = Options::parse();

    if args.verbose {
        SimpleLogger::new().init().unwrap();
    }

    let rt = Runtime::new().unwrap();

    let _ = rt.block_on(run(args.width, args.height));
}

async fn run(width: u32, height: u32) -> Result<(), impl std::error::Error> {
    let event_loop = EventLoopBuilder::new().build()?;

    event_loop.listen_device_events(winit::event_loop::DeviceEvents::Never);
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

    let window = WindowBuilder::new()
        .with_title("Shader-rs")
        .with_inner_size(winit::dpi::PhysicalSize::new(width, height))
        .build(&event_loop)?;

    let state_window_id = window.id();

    let mut state = State::new(window).await;

    event_loop.run(move |event, elwt| match event {
        Event::WindowEvent { event, window_id } if window_id == state_window_id => {
            match event {
                WindowEvent::CloseRequested => elwt.exit(),
                WindowEvent::Resized(physical_size) => state.resize(physical_size),
                _ => (),
            }
        }

        Event::AboutToWait => match state.render() {
            Ok(_) => {}
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                state.resize(state.size)
            }
            Err(wgpu::SurfaceError::OutOfMemory) => elwt.exit(),
            Err(wgpu::SurfaceError::Timeout) => log::warn!("Surface timeout"),
        },

        _ => (),
    })
}
