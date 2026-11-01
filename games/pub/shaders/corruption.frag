// The corruption event: the finished frame pixelates into coarse blocks, tears rows sideways and
// splits the colours, then settles back to normal. The engine supplies Texture (the frame), Strength
// (0 to 1: snaps in, holds, falls away), Progress (0 to 1 over the effect) and Step (changes 12 times
// a second).
#version 100
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
}
