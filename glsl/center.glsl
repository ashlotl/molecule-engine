#version 460

layout (set = 0, binding = 0) uniform Data {
    ivec2 window_dimensions;
} uniforms;

layout(location = 0) out vec4 f_color;

void main() {
    vec2 pos = gl_FragCoord.xy/uniforms.window_dimensions.xy-vec2(0.5, 0.5);
    float ret = pos.x*pos.x+pos.y*pos.y;
    f_color = vec4(ret);
}