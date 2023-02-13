#version 450

layout(location = 0) in vec2 in_uv;

layout(location = 0) out vec4 output_color;

void main()
{
    output_color = vec4(fract(in_uv), 0.0, 1.0);
}
