#version 330 core
out vec4 FragColor;

in vec2 tex_pos;

uniform vec2 viewport_size;
uniform vec2 tileset_size;

uniform sampler2D tileset;
uniform sampler2D scene_tiles;
uniform sampler2D front_scene_tiles_modifiers;
uniform sampler2D back_scene_tiles_modifiers;

void main()
{
    vec2 global_tile_uv = tex_pos * viewport_size;
    ivec2 tile_index = ivec2(global_tile_uv);
    vec2 tile_uv = global_tile_uv - tile_index;
    vec2 tile_size = vec2(1.0) / tileset_size;

    // calculate front tile modifier data
    vec4 front_modifiers = texelFetch(front_scene_tiles_modifiers, tile_index, 0).rgba;
    vec4 front_modifier_color = vec4(front_modifiers.xyz, 1.0);

    float front_modifier_flip = front_modifiers.a;
    vec2 front_tile_uv;
    if ((front_modifier_flip > 0.1 && front_modifier_flip < 0.3) || front_modifier_flip > 0.5) {
        front_tile_uv.x = 1.0 - tile_uv.x;
    }

    if ((front_modifier_flip > 0.3 && front_modifier_flip < 0.5) || front_modifier_flip > 0.5) {
        front_tile_uv.y = 1.0 - tile_uv.y;
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


    // show front tile if it isn't transparent, else show the back tile
    if (front_tile_color.a > 0.0) {
        FragColor = front_tile_color * front_modifier_color;
    } else {
        // calculate back tile modifier data
        vec4 back_modifiers = texelFetch(back_scene_tiles_modifiers, tile_index, 0).rgba;
        vec4 back_modifier_color = vec4(back_modifiers.xyz, 1.0);

        float back_modifier_flip = back_modifiers.a;
        vec2 back_tile_uv;
        if ((back_modifier_flip > 0.1 && back_modifier_flip < 0.3) || back_modifier_flip > 0.5) {
            back_tile_uv.x = 1.0 - tile_uv.x;
        }

        if ((back_modifier_flip > 0.3 && back_modifier_flip < 0.5) || back_modifier_flip > 0.5) {
            back_tile_uv.y = 1.0 - tile_uv.y;
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

        FragColor = back_tile_color * back_modifier_color;
    }


}

