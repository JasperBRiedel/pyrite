#version 330 core
layout (location = 0) in vec2 in_quad_pos;
layout (location = 1) in vec2 in_tex_pos;

out vec2 tex_pos;

void main()
{
	gl_Position = vec4(in_quad_pos, 0.0, 1.0);
	tex_pos = in_tex_pos;
}
