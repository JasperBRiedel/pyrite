#version 330 core
out vec4 FragColor;

in vec2 tex_pos;

uniform uvec2 viewport_size;
uniform uvec2 tileset_size;
uniform ivec2 tile_size;
uniform uvec2 framebuffer_size;
uniform uint scale;

uniform sampler2D tileset;
uniform sampler2D scene_tiles;
uniform sampler2D front_scene_tiles_modifiers;
uniform sampler2D back_scene_tiles_modifiers;

ivec2 calculate_flip(float flip_modifier, ivec2 pixel_pos, ivec2 pixel_range) {
    ivec2 out_pixel_pos = pixel_pos;

    if ((flip_modifier > 0.1 && flip_modifier < 0.3) || flip_modifier > 0.5) {
        out_pixel_pos.x = pixel_range.x - 1 - pixel_pos.x;
    }

    if ((flip_modifier > 0.3 && flip_modifier < 0.5) || flip_modifier > 0.5) {
        out_pixel_pos.y = pixel_range.y - 1 - pixel_pos.y;
    }

    return out_pixel_pos;
}

void main()
{
    ivec2 pixel_pos = ivec2((ivec2(framebuffer_size) / float(scale)) * tex_pos);
    ivec2 tile_pixel_pos = ivec2(mod(pixel_pos, tile_size));
    ivec2 tile_pos = pixel_pos / tile_size;

    ivec4 tile_tex_offset = ivec4(texelFetch(scene_tiles, tile_pos, 0));

    vec4 front_tile_modifiers = texelFetch(front_scene_tiles_modifiers, tile_pos, 0);
    vec4 front_modifier_color = vec4(front_tile_modifiers.xyz, 1.0);

    ivec2 front_tile_pixel_pos = calculate_flip(
        front_tile_modifiers.w, 
        tile_pixel_pos,
        tile_size
    );

    vec4 front_tile_color = texelFetch(
        tileset, 
        tile_tex_offset.xy * ivec2(tile_size) + front_tile_pixel_pos, 
        0
    );

    if (length(front_tile_color) > 0) {
        // fix for tiles that have transparent pixels, but still have the pixel data.
        FragColor = vec4(front_tile_color.rgb * front_modifier_color.rgb * front_tile_color.a,
                1.0);
    } else {

        vec4 back_tile_modifiers = texelFetch(back_scene_tiles_modifiers, tile_pos, 0);
        vec4 back_modifier_color = vec4(back_tile_modifiers.xyz, 1.0);
        ivec2 back_tile_pixel_pos = calculate_flip(
            back_tile_modifiers.w, 
            tile_pixel_pos,
            tile_size
        );

        vec4 back_tile_color = texelFetch(
            tileset, 
            tile_tex_offset.zw * ivec2(tile_size) + back_tile_pixel_pos, 
            0
        );

        // fix for tiles that have transparent pixels, but still have the pixel data.
        FragColor = vec4(back_tile_color.rgb * back_modifier_color.rgb * back_tile_color.a,
                1.0);
    }
}

