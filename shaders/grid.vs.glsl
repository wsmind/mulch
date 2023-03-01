#version 450

#include <view.h>

layout(location = 0) out vec3 out_world_position;

vec2[] positions = {
    vec2(-100.0, -100.0),
    vec2(-100.0, 100.0),
    vec2(100.0, -100.0),
    vec2(100.0, 100.0),
};

void main()
{
    vec4 position = vec4(positions[gl_VertexIndex], 0.0, 1.0);
    out_world_position = position.xyz;
    gl_Position = projection_matrix * view_matrix * position;
}
