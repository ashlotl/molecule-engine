#version 460

layout (set = 0, binding = 0) uniform Data {
    ivec2 window_dimensions;
    uint time;
} uniforms;

layout(location = 0) out vec4 f_color;

void main() {
    vec2 norm_coordinates = (gl_FragCoord.xy/uniforms.window_dimensions - vec2(0.5, 0.5));
    vec2 c = (norm_coordinates) * 2.0 - vec2(0.55, 0.55);
    // c*=vec2(cos(float(uniforms.time)/100), sin(float(uniforms.time)/100));

    vec2 z = vec2(0.0, 0.0);
    float i;
    for (i = 0.0; i < 1.0; i += 0.005) {
        z = vec2(
            z.x * z.x - z.y * z.y + c.x,
            z.y * z.x + z.x * z.y + c.y
        );
        z.x*=cos(uniforms.time/300.0);

        if (length(z) > 4.0) {
            break;
        }
    }

    float a = 1-i;

    f_color = vec4(
        vec3(i),
        1.0
    );
}