#version 450

layout(location=0) in vec2 i_Quad_Vertex;
layout(location=1) in vec2 i_Pos;
layout(location=2) in vec2 i_Scale;
layout(location=3) in vec2 i_Atlas_Pos;
layout(location=4) in vec2 i_Atlas_Size;
layout(location=5) in vec2 i_Rotate_Origin;
layout(location=6) in float i_Rotation;
layout(location=7) in uint i_Atlas_Layer;
layout(location=8) in uint i_Is_Hi_Dpi;

layout(set = 0, binding = 0) uniform Globals {
    vec2 u_Scale;
    vec2 u_Atlas_Scale;
};

layout(location=0) out vec3 o_Uv;

vec2 rotate_around(vec2 point, vec2 origin, float angle) {
    float cos_theta = cos(angle);
    float sin_theta = sin(angle);
    float dx = point.x - origin.x;
    float dy = point.y - origin.y;

    return vec2(
        (cos_theta * dx) - (sin_theta * dy) + origin.x,
        (sin_theta * dx) + (cos_theta * dy) + origin.y
    );
}

void main() {
    o_Uv = vec3((i_Atlas_Pos + (i_Quad_Vertex * i_Atlas_Size)) * u_Atlas_Scale, i_Atlas_Layer);

    if (i_Rotation == 0.0) {
        vec2 pos = (i_Pos + (i_Quad_Vertex * i_Atlas_Size * i_Scale)) * u_Scale;
        gl_Position = vec4(pos.x - 1.0, pos.y + 1.0, 0.0, 1.0);
    } else {
        vec2 rotated_offset = rotate_around(i_Quad_Vertex * i_Atlas_Size * i_Scale, i_Rotate_Origin, i_Rotation);

        vec2 pos = (i_Pos + rotated_offset) * u_Scale;
        gl_Position = vec4(pos.x - 1.0, pos.y + 1.0, 0.0, 1.0);
    }
}