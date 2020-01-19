#version 330 core
out vec4 FragColor;

in vec2 tex_pos;

uniform float time;
uniform vec2 framebuffer_size;
uniform vec2 viewport_size;
uniform vec2 tileset_size;

uniform sampler2D tileset;
uniform sampler2D scene_tiles;
uniform sampler2D scene_tiles_modifiers;

void main()
{
    vec2 global_tile_uv = tex_pos * viewport_size;
    ivec2 tile_index = ivec2(global_tile_uv);
    vec2 tile_uv = global_tile_uv - tile_index;

    vec4 modifiers = texelFetch(scene_tiles_modifiers, tile_index, 0).rgba;
    float modifier_flip = modifiers.a;
    vec4 modifier_color = vec4(modifiers.xyz, 1.0);

    if ((modifier_flip > 0.1 && modifier_flip < 0.3) || modifier_flip > 0.5) {
        tile_uv.x = 1.0 - tile_uv.x;
    }

    if ((modifier_flip > 0.3 && modifier_flip < 0.5) || modifier_flip > 0.5) {
        tile_uv.y = 1.0 - tile_uv.y;
    }

    vec2 tile_size = vec2(1.0) / tileset_size;
    vec2 tile_offset = texelFetch(scene_tiles, tile_index, 0).xy;
    vec2 tileset_uv = tile_size * tile_offset + tile_size * tile_uv;

    vec4 tile_color = texture(tileset, tileset_uv);

    FragColor = tile_color * modifier_color;
}

