#version 450

layout(location = 0) in vec3 in_world_position;

layout(location = 0) out vec4 output_color;

void main()
{
    float width = 1.5 * max(abs(dFdx(in_world_position.x)), abs(dFdy(in_world_position.x)));
    float alpha = smoothstep(width, 0.0, abs(fract(in_world_position.x + 0.5) - 0.5)) * (1.0 - width);

    width = 1.5 * max(abs(dFdx(in_world_position.y)), abs(dFdy(in_world_position.y)));
    alpha += smoothstep(width, 0.0, abs(fract(in_world_position.y + 0.5) - 0.5)) * (1.0 - width);

    alpha = clamp(alpha, 0.0, 1.0);

    output_color = vec4(0.1 + step(abs(in_world_position.y), 0.01) * 0.3, 0.1 + step(abs(in_world_position.x), 0.01) * 0.3, 0.1, alpha);
}
