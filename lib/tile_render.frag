#version 330 core
out vec4 FragColor;

in vec2 tex_pos;

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
    vec2 tile_size = vec2(1.0) / tileset_size;

    // calculate modifier data
    vec4 modifiers = texelFetch(scene_tiles_modifiers, tile_index, 0).rgba;
    vec4 modifier_color = vec4(modifiers.xyz, 1.0);

    float modifier_flip = modifiers.a;
    if ((modifier_flip > 0.1 && modifier_flip < 0.3) || modifier_flip > 0.5) {
        tile_uv.x = 1.0 - tile_uv.x;
    }

    if ((modifier_flip > 0.3 && modifier_flip < 0.5) || modifier_flip > 0.5) {
        tile_uv.y = 1.0 - tile_uv.y;
    }

    // calculate tile data
    vec4 tile_offset = texelFetch(scene_tiles, tile_index, 0);

    // calculate front tile colour
    vec2 front_tile_offset = tile_offset.rg;
    vec4 front_tile_color; 
    if (front_tile_offset.x <= -2.0) { // fill
        front_tile_color = vec4(1.0);
    } else if (front_tile_offset.x <= -1.0) { // none
        front_tile_color = vec4(0.0);
    } else {
        vec2 front_tileset_uv = tile_size * front_tile_offset + tile_size * tile_uv;
        front_tile_color = texture(tileset, front_tileset_uv);
    }

    // calculate back tile colour
    vec2 back_tile_offset = tile_offset.ba;
    vec4 back_tile_color; 
    if (back_tile_offset.x <= -2.0) { // fill
        back_tile_color = vec4(1.0);
    } else if (back_tile_offset.x <= -1.0) { // none
        back_tile_color = vec4(0.0);
    } else {
        vec2 back_tileset_uv = tile_size * back_tile_offset + tile_size * tile_uv;
        back_tile_color = texture(tileset, back_tileset_uv);
    }

    // show front tile if it isn't transparent, else show the back tile
    if (front_tile_color.a > 0.0) {
        FragColor = front_tile_color * modifier_color;
    } else {
        FragColor = back_tile_color * modifier_color;
    }


}

