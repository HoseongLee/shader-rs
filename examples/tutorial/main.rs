use clap::Parser;
use simple_logger::SimpleLogger;
use tokio::runtime::Runtime;

use shader_rs::run;

#[derive(Parser, Debug)]
#[command(version)]
struct Options {
    #[arg(long, default_value_t = 512)]
    width: u32,

    #[arg(long, default_value_t = 512)]
    height: u32,

    #[arg(long)]
    verbose: bool,
}

fn main() {
    let args = Options::parse();

    if args.verbose {
        SimpleLogger::new().init().unwrap();
    }

    let rt = Runtime::new().unwrap();

    let frag_shader_desc = wgpu::include_wgsl!("shader.wgsl");
    let vert_shader_desc = wgpu::include_wgsl!("vertex.wgsl");

    let _ = rt.block_on(run(
        args.width,
        args.height,
        frag_shader_desc,
        vert_shader_desc,
    ));
}
