@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> @builtin(position) vec4<f32> {
    let x = f32(in_vertex_index % 2u) * 4. - 1.;
    let y = f32(in_vertex_index / 2u) * 4. - 1.;
    return vec4<f32>(x, y, 0., 1.);
}
