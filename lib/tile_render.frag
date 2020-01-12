#version 330 core
out vec4 FragColor;

in vec2 tex_pos;

uniform float time;

void main()
{
    vec3 col = 0.5 + 0.5*cos(time+tex_pos.xyx+vec3(0,2,4));
    FragColor = vec4(col, 1.0);
}
