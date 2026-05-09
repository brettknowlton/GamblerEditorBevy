#import bevy_sprite::mesh2d_vertex_output::VertexOutput

struct PixelEffectUniform {
    pixel_size: f32,
    color_levels: f32,
    dither_strength: f32,
    scanline_strength: f32,
    palette_enabled: f32,
    _pad_a: f32,
    _pad_b: f32,
    _pad_c: f32,
};

@group(#{MATERIAL_BIND_GROUP}) @binding(0)
var source_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(1)
var source_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(2)
var<uniform> effect: PixelEffectUniform;
@group(#{MATERIAL_BIND_GROUP}) @binding(3)
var palette_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(4)
var palette_sampler: sampler;

fn bayer_4x4(pixel: vec2<i32>) -> f32 {
    let x = pixel.x & 3;
    let y = pixel.y & 3;

    let idx = y * 4 + x;
    let table = array<f32, 16>(
        0.0, 8.0, 2.0, 10.0,
        12.0, 4.0, 14.0, 6.0,
        3.0, 11.0, 1.0, 9.0,
        15.0, 7.0, 13.0, 5.0
    );

    return table[idx] / 16.0;
}

fn quantize(rgb: vec3<f32>, levels: f32, dither: f32) -> vec3<f32> {
    let span = max(levels - 1.0, 1.0);
    return floor(rgb * span + dither) / span;
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let tex_dims = vec2<f32>(textureDimensions(source_texture));
    let pixel_size = max(effect.pixel_size, 1.0);

    let snapped_px = floor(mesh.uv * tex_dims / pixel_size) * pixel_size;
    let snapped_uv = (snapped_px + 0.5 * pixel_size) / tex_dims;

    var color = textureSample(source_texture, source_sampler, snapped_uv);
    var rgb = color.rgb;

    let dither_value = (bayer_4x4(vec2<i32>(snapped_px)) - 0.5) * effect.dither_strength;
    rgb = quantize(rgb, effect.color_levels, dither_value);

    let scan = 0.5 + 0.5 * sin((snapped_uv.y * tex_dims.y) * 3.14159265);
    rgb *= 1.0 - (effect.scanline_strength * scan * 0.35);

    if effect.palette_enabled > 0.5 {
        let luma = dot(rgb, vec3<f32>(0.299, 0.587, 0.114));
        let remapped = textureSample(palette_texture, palette_sampler, vec2<f32>(luma, 0.5));
        rgb = remapped.rgb;
    }

    color = vec4<f32>(rgb, color.a);

    return color;
}
