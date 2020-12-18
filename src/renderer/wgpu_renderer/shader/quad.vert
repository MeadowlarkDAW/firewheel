#version 450

layout(location = 0) in vec2 a_Quad_Vertex;
layout(location = 1) in vec2 a_Pos;
layout(location = 2) in vec2 a_Scale;
layout(location = 3) in vec4 a_Color;
layout(location = 4) in vec4 a_BorderColor;
layout(location = 5) in float a_BorderRadius;
layout(location = 6) in float a_BorderWidth;

layout (set = 0, binding = 0) uniform Globals {
    mat4 u_Transform;
};

layout(location = 0) out vec4 o_Color;
layout(location = 1) out vec4 o_BorderColor;
layout(location = 2) out vec2 o_Pos;
layout(location = 3) out vec2 o_Scale;
layout(location = 4) out float o_BorderRadius;
layout(location = 5) out float o_BorderWidth;

void main() {
    float a_BorderRadius = min(
        a_BorderRadius,
        min(a_Scale.x, a_Scale.y) / 2.0
    );

    mat4 a_Transform = mat4(
        vec4(a_Scale.x + 1.0, 0.0, 0.0, 0.0),
        vec4(0.0, a_Scale.y + 1.0, 0.0, 0.0),
        vec4(0.0, 0.0, 1.0, 0.0),
        vec4(a_Pos - vec2(0.5, 0.5), 0.0, 1.0)
    );

    o_Color = a_Color;
    o_BorderColor = a_BorderColor;
    o_Pos = a_Pos;
    o_Scale = a_Scale;
    o_BorderRadius = a_BorderRadius;
    o_BorderWidth = a_BorderWidth;

    gl_Position = u_Transform * a_Transform * vec4(a_Quad_Vertex, 0.0, 1.0);
}