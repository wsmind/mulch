#version 450

layout(location = 0) in vec3 in_world_position;

layout(location = 0) out vec4 output_color;

void main()
{
    float width = 1.5 * max(abs(dFdx(in_world_position.x)), abs(dFdy(in_world_position.x)));
    float alpha = smoothstep(width, 0.0, abs(fract(in_world_position.x + 0.5) - 0.5)) * (1.0 - width);
    float green = step(abs(in_world_position.x), width);

    width = 1.5 * max(abs(dFdx(in_world_position.y)), abs(dFdy(in_world_position.y)));
    alpha += smoothstep(width, 0.0, abs(fract(in_world_position.y + 0.5) - 0.5)) * (1.0 - width);
    float red = step(abs(in_world_position.y), width);

    alpha = clamp(alpha, 0.0, 1.0);

    float intensity = 0.12;
    float color_intensity = 0.1;
    output_color = vec4(intensity + red * color_intensity, intensity + green * color_intensity, intensity, alpha);
}
