#version 450

layout(location=0) in vec2 i_Quad_Vertex;
layout(location=1) in vec2 i_Pos;
layout(location=2) in vec2 i_Size;
layout(location=3) in vec2 i_Atlas_Pos;
layout(location=4) in vec2 i_Atlas_Size;
layout(location=5) in uint i_Atlas_Layer;

layout(set = 0, binding = 0) uniform Globals {
    vec2 u_Scale;
    vec2 u_Atlas_Scale;
};

layout(location=0) out vec3 o_Uv;

void main() {
    o_Uv = vec3((i_Atlas_Pos + (i_Quad_Vertex * i_Atlas_Size)) * u_Atlas_Scale, i_Atlas_Layer);

    vec2 o_Pos = (i_Pos + (i_Quad_Vertex * i_Size)) * u_Scale;
    gl_Position = vec4(o_Pos.x - 1.0, o_Pos.y + 1.0, 0.0, 1.0);
}