#version 450

#include <view.h>

layout(location = 0) in vec3 in_position;
layout(location = 1) in vec3 in_normal;

layout(location = 0) out vec3 out_world_position;
layout(location = 1) out vec3 out_normal;

void main()
{
    vec4 position = vec4(in_position, 1.0);
    out_world_position = position.xyz;
    out_normal = normalize(in_normal);
    gl_Position = projection_matrix * view_matrix * position;
}
