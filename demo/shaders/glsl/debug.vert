#version 450
#extension GL_ARB_separate_shader_objects : enable

#include "debug.glsl"

// @[semantic("POSITION")]
layout(location = 0) in vec3 in_pos;

// @[semantic("COLOR")]
layout(location = 1) in vec4 in_color;

layout(location = 0) out vec4 out_color;

void main() {
    out_color = in_color;
    gl_Position = per_frame_data.view_proj * vec4(in_pos.x, in_pos.y, in_pos.z, 1.0);
}