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
    if (tile_index.x == 0 && tile_index.y == 0) {
        FragColor = texture(tileset, uv);
    } else if (tile_index.x == 1 && tile_index.y == 1) {
        vec2 tileset_uv_offset = texelFetch(scene_tiles, tile_index, 0).xy;
        FragColor = vec4(tileset_uv_offset.xy, 0.0, 1.0);
    } else {
        FragColor = vec4(uv.xyx, 1.0);
    }
}

void main()
{
    vec2 framebuffer_pos = tex_pos * framebuffer_size;

    if (tex_pos.x >= 0.5 && tex_pos.y >= 0.5) {
        FragColor = vec4(0.1, 0.1, 1.1, 1.0);
    } else {
        vec2 global_tile_uv = tex_pos * viewport_size;
        ivec2 tile_index = ivec2(global_tile_uv);
        vec2 tile_uv = global_tile_uv - tile_index;
        sample_tile(tile_index, tile_uv);
    }
}

