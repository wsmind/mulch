#version 450

#include <ui/ui.h>

layout(location = 0) in vec2 in_uv;
layout(location = 1) in vec4 in_color;

layout(location = 0) out vec4 out_color;

void main() {
    vec4 color = texture(sampler2D(color_texture, color_sampler), in_uv);
    out_color = in_color * color;
}
