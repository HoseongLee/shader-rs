struct Uniforms {
    time: f32,
    resolution: vec2<f32>,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@fragment
fn fs_main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    var snow = 0.0;

    let t = uniforms.time;
    let uv = pos.xy / uniforms.resolution.x ;
    let gradient = pos.y / uniforms.resolution.y + 0.3;

    let c = cos(t * 2.5);
    let s = sin(t * 2.5);

    for (var k = 0.7; k < 6.0; k = k + 1.3) {
        let x = cos(k + t);
        let y = sin(k + t);

        for (var i = 1.0; i < 5.0; i = i + 1.0) {
            let cellSize = 2.0 + 2.0 * i;

            let uvShifted = uv - vec2(sin(k + t) * 0.05, t - cos(k + t) * 0.3) / i;
            let uvStep = ceil(uvShifted * cellSize - 0.5) / cellSize;

            if fract(sin(dot(uvStep, vec2(17., 19.))) * 129.) < 0.1 {
                let d = distance(uvStep + vec2(x * c, y * s) * 0.7 / cellSize, uvShifted) * cellSize;

                snow += clamp(d, 0., 1.);
            }
        }
    }

    return vec4(snow, snow, snow, 1.0) + 0.2 * gradient * vec4(0.4, 0.8, 1.0, 0.0);
}
