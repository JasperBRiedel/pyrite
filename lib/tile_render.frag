#version 330 core
out vec4 FragColor;

in vec2 tex_pos;

uniform float time;
uniform vec2 framebuffer_size;

void main()
{
    vec2 framebuffer_pos = tex_pos * framebuffer_size;

    vec3 col = 0.5 + 0.5*cos(time+tex_pos.xyx+vec3(0,2,4));

    if (framebuffer_pos.x <= 50.0) {
        FragColor = vec4(1.0, 0.0, 0.0, 1.0);
    } else {
        FragColor = vec4(col, 1.0);
    }
}
