struct Uniforms {
    time: f32,
    resolution: vec2<f32>,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@fragment
fn fs_main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    let graident = pos.y / uniforms.resolution.y * 0.4;

    var snow = 0.0;

    for (var k = 0.0; k < 6.0; k=k+1.0) {
        for (var i = 0.0; i < 2.0; i=i+1.0) {
            let cellSize = 2.0 + i * 3.0;
            let downSpeed = 0.3 + (sin(uniforms.time * 0.4 + k + i * 20.0) + 1.0) * 0.00008;

            let uv = (pos.xy / uniforms.resolution.x);// + vec2(0.01*sin((uniforms.time+k*6185.0)*0.6+i)*(5.0/i), -downSpeed*(uniforms.time+k*1352.0)*(1.0/i));
            let uvStep = ceil(uv * cellSize - 0.5) / cellSize;

            let x = fract(sin(dot(uvStep, vec2(12.9898+k*12.0,78.233+k*315.156)))* 43758.5453+k*12.0) - 0.5;
            let y = fract(sin(dot(uvStep, vec2(62.2364+k*23.0,94.674+k*95.0)))* 62159.8432+k*12.0) - 0.5;

            let randomMagnitude1 = sin(uniforms.time * 2.5) * 0.7 / cellSize;
            let randomMagnitude2 = cos(uniforms.time * 2.5) * 0.7 / cellSize;

            let d = 5.0 * distance(uvStep + vec2(x * sin(y), y) * randomMagnitude1 + vec2(y, x) * randomMagnitude2, uv.xy);

            let omiVal = fract(sin(dot(uvStep, vec2(32.4691, 94.615))) * 31572.1684);

            if (omiVal < 0.08) {
                let newd = (x+1.0)*0.4*clamp(1.9-d*(15.0+(x*6.3))*(cellSize/1.4),0.0,1.0);
                snow += newd;
            }
        }
    }

    return vec4(snow) + graident * vec4(0.4, 0.8, 1.0, 0.0);
}
