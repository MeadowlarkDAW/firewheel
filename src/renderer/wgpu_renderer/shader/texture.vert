#version 450

layout(location=0) in vec2 a_Quad_Vertex;
layout(location=1) in vec2 a_Pos;
layout(location=2) in vec2 a_Atlas_Pos;
layout(location=3) in vec2 a_Atlas_Size;
layout(location=4) in vec2 a_Rotate_Origin;
layout(location=5) in float a_Rotation;
layout(location=6) in uint a_Atlas_Layer;
layout(location=7) in uint a_Is_Hi_Dpi;

layout(set = 0, binding = 0) uniform Globals {
    mat4 u_Transform;
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
    o_Uv = vec3((a_Atlas_Pos + (a_Quad_Vertex * a_Atlas_Size)) * u_Atlas_Scale, a_Atlas_Layer);

    if (a_Rotation == 0.0) {
        gl_Position = u_Transform * vec4(a_Pos + (a_Quad_Vertex * a_Atlas_Size), 0.0, 1.0);
    } else {
        vec2 offset = rotate_around(a_Quad_Vertex * a_Atlas_Size, a_Rotate_Origin, a_Rotation);

        gl_Position = u_Transform * vec4(a_Pos + offset, 0.0, 1.0);
    }
}