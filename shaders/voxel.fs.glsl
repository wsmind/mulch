#version 450

layout(location = 0) in vec3 in_world_position;
layout(location = 1) in vec3 in_normal;

layout(location = 0) out vec4 output_color;

const vec3 light = normalize(vec3(1.0));

void main()
{
    vec3 normal = normalize(in_normal);
    float diffuse = dot(normal, light) * 0.5 + 0.5;

    output_color = vec4(diffuse.xxx * 0.7, 1.0);
}
