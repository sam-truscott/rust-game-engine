//! The corruption event: for a second the finished frame is drawn through a shader that pixelates it
//! into coarse blocks, tears rows sideways and splits the colours, then settles back to normal.

use macroquad::prelude::*;

const VERTEX: &str = r#"#version 100
attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 color0;
varying lowp vec2 uv;
varying lowp vec4 color;
uniform mat4 Model;
uniform mat4 Projection;
void main() {
    gl_Position = Projection * Model * vec4(position, 1.0);
    color = color0 / 255.0;
    uv = texcoord;
}"#;

const FRAGMENT: &str = r#"#version 100
precision highp float;
varying lowp vec2 uv;
varying lowp vec4 color;
uniform sampler2D Texture;
uniform float Strength;
uniform float Step;

float hash(vec2 p) {
    return fract(sin(dot(p, vec2(127.1, 311.7)) + Step * 17.3) * 43758.5453);
}

void main() {
    // From 320 blocks across (the art's own grid) down to 20 at full strength.
    float across = mix(320.0, 20.0, Strength);
    vec2 cells = vec2(across, across * 9.0 / 16.0);
    vec2 block = floor(uv * cells);
    vec2 centre = (block + 0.5) / cells;

    float row = hash(vec2(block.y, 1.0));
    float tear = step(0.8, row) * (hash(vec2(block.y, 2.0)) - 0.5) * 0.3 * Strength;
    vec2 at = vec2(centre.x + tear, centre.y);
    float split = 0.02 * Strength;

    vec3 rgb = vec3(
        texture2D(Texture, at + vec2(split, 0.0)).r,
        texture2D(Texture, at).g,
        texture2D(Texture, at - vec2(split, 0.0)).b);

    float flip = step(0.98 - 0.1 * Strength, hash(block));
    rgb = mix(rgb, 1.0 - rgb, flip * Strength);
    gl_FragColor = vec4(rgb, 1.0) * color;
}"#;

pub struct Corruption {
    material: Material,
}

impl Corruption {
    pub fn new() -> Self {
        let material = load_material(
            ShaderSource::Glsl {
                vertex: VERTEX,
                fragment: FRAGMENT,
            },
            MaterialParams {
                uniforms: vec![
                    UniformDesc::new("Strength", UniformType::Float1),
                    UniformDesc::new("Step", UniformType::Float1),
                ],
                ..Default::default()
            },
        )
        .unwrap_or_else(|error| panic!("corruption shader: {error}"));
        Corruption { material }
    }

    /// Makes everything drawn until `end` go through the effect; `progress` is 0 to 1 over the event.
    pub fn begin(&self, progress: f32) {
        // Snaps in, holds, then falls away; the pattern changes 12 times a second.
        let strength = (progress * std::f32::consts::PI).sin().powf(0.5);
        self.material.set_uniform("Strength", strength);
        self.material.set_uniform("Step", (progress * 12.0).floor());
        gl_use_material(&self.material);
    }

    pub fn end(&self) {
        gl_use_default_material();
    }
}
