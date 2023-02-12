#ifndef _UI_H_
#define _UI_H_

layout(binding = 0) uniform texture2D color_texture;
layout(binding = 1) uniform sampler color_sampler;

layout(binding = 2) uniform viewport_constants
{
    vec4 viewport_transform; // xy: scale, zw: offset
};

#endif // _UI_H_
