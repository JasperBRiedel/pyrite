#version 330 core
out vec4 FragColor;

in vec2 tex_pos;

uniform float time;
uniform vec2 framebuffer_size;
uniform vec2 viewport_size;

uniform sampler2D tileset;
uniform sampler2D scene_tiles;
uniform sampler2D scene_tiles_modifiers;

void sample_tile(ivec2 tile_index, vec2 uv) {
    vec2 tileset_uv_offset = texelFetch(scene_tiles, tile_index, 0).xy;
    vec4 modifiers = texelFetch(scene_tiles_modifiers, tile_index, 0).rgba;
    FragColor = vec4(modifiers.xyz, 1.0);
}

void main()
{
    vec2 global_tile_uv = tex_pos * viewport_size;
    ivec2 tile_index = ivec2(global_tile_uv);
    vec2 tile_uv = global_tile_uv - tile_index;
    sample_tile(tile_index, tile_uv);
}

