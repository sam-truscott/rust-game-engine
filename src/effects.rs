//! Whole-screen effects: a script says `screenEffect("name", seconds)` and for that long the finished
//! frame is drawn through the fragment shader `shaders/<name>.frag` of the game. The shader gets
//! `Texture` (the frame), `Strength` (0 to 1: snaps in, holds, falls away), `Progress` (0 to 1 over
//! the effect) and `Step` (changes 12 times a second, to vary a pattern).

use std::cell::RefCell;
use std::collections::HashMap;

use game::assets;
use game::world::ScreenEffect;
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

const STEPS_PER_SECOND: f32 = 12.0;

#[derive(Default)]
pub struct ScreenEffects {
    /// Compiled shaders by name; `None` for one that is missing or does not compile, said once.
    materials: RefCell<HashMap<String, Option<Material>>>,
}

impl ScreenEffects {
    fn material(&self, name: &str) -> Option<Material> {
        self.materials
            .borrow_mut()
            .entry(name.to_owned())
            .or_insert_with(|| {
                let path = format!("shaders/{name}.frag");
                let Some(fragment) = assets::text(&path) else {
                    eprintln!("[effect] missing {path}");
                    return None;
                };
                load_material(
                    ShaderSource::Glsl {
                        vertex: VERTEX,
                        fragment: &fragment,
                    },
                    MaterialParams {
                        uniforms: vec![
                            UniformDesc::new("Strength", UniformType::Float1),
                            UniformDesc::new("Progress", UniformType::Float1),
                            UniformDesc::new("Step", UniformType::Float1),
                        ],
                        ..Default::default()
                    },
                )
                .map_err(|error| eprintln!("[effect] {path}: {error}"))
                .ok()
            })
            .clone()
    }

    /// Makes everything drawn until `end` go through the effect. Returns whether it did, so that
    /// `end` is only called after a `begin` that worked.
    pub fn begin(&self, effect: &ScreenEffect) -> bool {
        let Some(material) = self.material(&effect.name) else {
            return false;
        };
        let strength = (effect.progress * std::f32::consts::PI).sin().powf(0.5);
        material.set_uniform("Strength", strength);
        material.set_uniform("Progress", effect.progress);
        material.set_uniform(
            "Step",
            (effect.progress * effect.seconds * STEPS_PER_SECOND).floor(),
        );
        gl_use_material(&material);
        true
    }

    pub fn end(&self) {
        gl_use_default_material();
    }
}
