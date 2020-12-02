#version 450

layout(location=0) in vec3 i_Uv;
layout(location=0) out vec4 o_Color;

layout(set = 0, binding = 1) uniform sampler u_Sampler;
layout(set = 1, binding = 0) uniform texture2DArray u_Texture;

void main() {
    o_Color = texture(sampler2DArray(u_Texture, u_Sampler), i_Uv);
}