#version 440

#extension GL_GOOGLE_include_directive : require

#include "structures.glsl"

layout(location = 0) in vec3 i_position;
layout(location = 1) in vec3 i_normal;
layout(location = 2) in vec3 i_tangent;
layout(location = 3) in vec2 i_coords;
layout(location = 4) in vec4 i_color;
layout(location = 5) in uint i_material;
#ifdef GPU_MODE
layout(location = 6) in uint i_object_idx;
#endif

layout(location = 0) out vec4 o_view_position;
layout(location = 1) out vec3 o_normal;
layout(location = 2) out vec3 o_tangent;
layout(location = 3) out vec2 o_coords;
layout(location = 4) out vec4 o_color;
layout(location = 5) flat out uint o_material;

layout(set = 1, binding = 0, std430) restrict readonly buffer ObjectOutputDataBuffer {
    ObjectOutputData object_output[];
};
layout(set = 3, binding = 0) uniform UniformBuffer {
    UniformData uniforms;
};

void main() {
    #ifdef CPU_MODE
    uint object_idx = gl_InstanceIndex;
    #else
    uint object_idx = i_object_idx;
    #endif

    ObjectOutputData data = object_output[object_idx];

    gl_Position = data.model_view_proj * vec4(i_position, 1.0);

    o_material = data.material_idx;

    o_view_position = data.model_view * vec4(i_position, 1.0);

    o_normal = data.inv_trans_model_view * i_normal;

    o_tangent = data.inv_trans_model_view * i_tangent;

    o_color = i_color;

    o_coords = i_coords;
}
