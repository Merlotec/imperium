#version 450

   const float MAXZ =  10000.0;
   const float MAX_REL_Z = 1.0;

   #extension GL_ARB_separate_shader_objects : enable

   const int BONES_PER_VERTEX = 4;
   const int MAX_BONES = 100;

   struct BoneData {
       int id;
       float weight;
   };

   struct BoneList {
       int count;
       mat4 data[MAX_BONES];
   };

   layout(location = 0) in vec3 position;
   layout(location = 1) in vec3 normal;
   layout(location = 2) in vec2 uv_Pos;
   layout(location = 3) in ivec4 bone_ids;
   layout(location = 4) in vec4 bone_weights;

   layout(push_constant) uniform Transform {
       mat4 model;
       mat4 view;
       mat4 projection;
   };
   layout(set = 0, binding = 0) uniform u_BoneList {
       BoneList bone_list;
   };

   layout(location = 0) out vec2 uv;
   layout(location = 1) out vec3 norm;

   void main() {
       uv = uv_Pos;

       mat4 bone_transform = mat4(1.0);
       for (int i = 0; i < BONES_PER_VERTEX; i++) {
           if (bone_list.count > i) {
               if (bone_ids[i] >= 0) {
                   mat4 trans = bone_list.data[bone_ids[i]] * bone_weights[i];
                   bone_transform *= trans;
               }
           } else {
               break;
           }
       }

       //mat4 local_transform = bone_transform;
       mat4 local_transform = model;

       mat4 camera_transform = projection * view;

       vec4 norm_calc = local_transform * vec4(normal, 0.0);
       norm = norm_calc.xyz;

       vec4 pos = camera_transform * local_transform * vec4(position, 1.0);

       float zpos = (pos.z / MAXZ) * MAX_REL_Z;
      // gl_Position = vec4(pos.xy, zpos, pos.w);
      gl_Position = pos;

   }