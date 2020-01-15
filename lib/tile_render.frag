#version 330 core
out vec4 FragColor;

in vec2 tex_pos;

uniform float time;
uniform vec2 framebuffer_size;
uniform vec2 viewport_size;

uniform sampler2D tileset; 

void main()
{
    vec2 framebuffer_pos = tex_pos * framebuffer_size;

    /* vec3 col = 0.5 + 0.5*cos(time+tex_pos.xyx+vec3(0,2,4)); */

    vec2 grid_size = framebuffer_size / viewport_size;

    vec2 intersect_grid = ivec2(framebuffer_pos) % ivec2(grid_size);
    if (intersect_grid.x == 0 || intersect_grid.y == 0) {
        FragColor = vec4(1.0, 0.0, 0.0, 1.0);
    } else {
        if (tex_pos.x >= .5 && tex_pos.y >= .5) {
            FragColor = vec4(1.0, 0.1, 0.1, 1.0);
        } else {
            FragColor = texture(tileset, tex_pos);
        }
    }
}

void draw_tile(uint x, uint y) {

}
