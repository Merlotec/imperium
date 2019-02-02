use crate::*;

use std::convert::AsRef;
use ai::*;
use std::ptr::*;
use std::mem;
use libc::*;
use std::rc::Rc;
use std::sync::Arc;
use specs::Builder;

use spatial::pipe::mesh::MeshRenderPipeline;
use spatial::material;

/// The basic component which can render a mesh to the screen.
/// This contains vertex buffer data as well as texture data.
/// It has the capability to use index buffers but currently meshes loaded from files do not contain index buffers.
pub struct BufferedMesh {

    /// The vertex buffer object which contains vertex information.
    /// It is contained within a 'Resource' structure.
    pub vertex_buffer: Res<buffer::Buffer>,

    /// The optional index buffer type.
    pub index_buffer: Option<Res<buffer::Buffer>>,

}

impl BufferedMesh {

    /// Creates a enw mesh component from a mesh object.
    /// This mesh can be loaded from a model file.
    pub fn new(mesh: &Mesh, device: &core::Device) -> BufferedMesh {
        return BufferedMesh::create(&mesh.vertices, Some(mesh.indices.as_slice()), device);
    }

    /// Creates a new mesh component from the specified raw vertex buffer, index buffer and texture.
    /// The scene and render objects are required in order to load the mesh properly.
    pub fn create(verts: &[ModelVertex], indices: Option<&[u32]>, device: &core::Device) -> BufferedMesh {
        let vertex_buffer: Res<buffer::Buffer> = Res::Val(buffer::Buffer::alloc_vertex(verts, device));
        let mut index_buffer: Option<Res<buffer::Buffer>> = None;
        if let Some(indices) = indices {
            index_buffer = Some(Res::Val(buffer::Buffer::alloc(indices, gfx::buffer::Usage::INDEX, gfx::memory::Properties::CPU_VISIBLE, device)));
        }
        return BufferedMesh { vertex_buffer, index_buffer };
    }


    /// Creates a new BufferedMesh from the specified vertex buffer, index buffer and texture buffer.
    /// The scene and graphics object are needed to create the descriptor set that properly represents the scene.
    pub fn from_raw_buffers(vertex_buffer: Res<buffer::Buffer>, index_buffer: Option<Res<buffer::Buffer>>) -> Self {
        return BufferedMesh { vertex_buffer, index_buffer };
    }

    pub fn render(&mut self, transform: render::RenderTransform, materials_desc: &pipeline::DescriptorSet, pipeline: &mut spatial::pipe::mesh::MeshRenderPipeline, encoder: &mut command::Encoder) {
        let index_buffer: Option<&buffer::Buffer> = {
            if let Some(ibuf) = self.index_buffer.as_ref() {
                Some(ibuf.as_ref())
            } else {
                None
            }
        };
        let vertex_input = pipeline::VertexInput { vertex_buffer: &self.vertex_buffer, index_buffer };
        pipeline.render(&vertex_input, materials_desc, transform, encoder);
    }
}

impl specs::Component for BufferedMesh {
    type Storage = specs::VecStorage<Self>;
}

impl scene::ComponentOf<spatial::Spatial> for BufferedMesh {}

/*
impl spatial::BatchRenderComponent for BufferedMesh {

    fn render_batch(&mut self, transforms: &[Matrix4f], cycle: &mut spatial::RenderCycle) {

        let mut render_transforms: Vec<render::RenderTransform> = Vec::with_capacity(transforms.len());

        for transform in transforms {
            render_transforms.push(render::RenderTransform::new(*transform, cycle.scene.camera.get_view_matrix(), cycle.scene.camera.get_projection_matrix()));
        }
        let index_buffer: Option<&buffer::Buffer> = {
            if let Some(ibuf) = self.index_buffer.as_ref() {
                Some(ibuf.as_ref())
            } else {
                None
            }
        };
        let vertex_input = pipeline::VertexInput { vertex_buffer: &self.vertex_buffer, index_buffer };
        cycle.scene.render_mesh_batch(&vertex_input, &self.texture_input_set, &render_transforms, cycle.graphics, cycle.encoder);
    }

}
*/

pub trait FromAiVec3f {

    unsafe fn from_ai(ai_vec: AiVector3D) -> Self;

}

pub trait FromAiVec2f {

    unsafe fn from_ai(ai_vec: AiVector2D) -> Self;

}

pub trait FromAiMat4f {

    unsafe fn from_ai(ia_mat: AiMatrix4x4) -> Self;

}

impl FromAiVec3f for Vector3f {

    unsafe fn from_ai(ai_vec: AiVector3D) -> Vector3f {
        return Vector3f { x: ai_vec.x, y: ai_vec.y, z: ai_vec.z };
    }

}

impl FromAiVec2f for Vector2f {

    unsafe fn from_ai(ai_vec: AiVector2D) -> Vector2f {
        return Vector2f { x: ai_vec.x, y: ai_vec.y };
    }

}

impl FromAiMat4f for Matrix4f {

    unsafe fn from_ai(ai_mat: AiMatrix4x4) -> Self {

        return Matrix4f {
            x: Vector4f::new(ai_mat.a1, ai_mat.a2, ai_mat.a3, ai_mat.a4),
            y: Vector4f::new(ai_mat.b1, ai_mat.b2, ai_mat.b3, ai_mat.b4),
            z: Vector4f::new(ai_mat.c1, ai_mat.c2, ai_mat.c3, ai_mat.c4),
            w: Vector4f::new(ai_mat.d1, ai_mat.d2, ai_mat.d3, ai_mat.d4)
        }

    }

}

#[derive(Copy, Clone)]
pub struct ModelVertex {

    pub pos: Vector3f,
    pub normal: Vector3f,
    pub uv: Vector2f,
    pub bone_ids: Vector4i,
    pub bone_weights: Vector4f,

}

impl ModelVertex {

    pub fn new(pos: Vector3f, normal: Vector3f, uv: Vector2f) -> ModelVertex {
        return ModelVertex {
            pos,
            normal,
            uv,
            bone_ids: Vector4i::new(-1, -1, -1, -1),
            bone_weights: Vector4f::zero(),
        };
    }

    pub fn add_bone(&mut self, id: i32, weight: f32) {
        for i in 0..4 {
            if self.bone_ids[i] == -1 {
                self.bone_ids[i] = id;
                self.bone_weights[i] == weight;
                return;
            }
        }
    }

}

#[derive(Clone)]
pub struct Mesh {

    pub vertices: Vec<ModelVertex>,
    pub indices: Vec<u32>,
    pub material_index: usize,
    pub skeleton: Skeleton,
    pub ai_mesh: *mut AiMesh,

}

impl Mesh {

    pub fn from_file(path: &str, mesh_index: usize) -> Result<Self, &'static str> {
        if let Ok(model) = Model::from_file(path) {
            return model.meshes.get(mesh_index).cloned().ok_or("The index specified does not exist.");
        }
        return Err("Failed to load mesh from path");
    }

    /**
    This function contains some very dangerous code, especially by rust standards.
    We have to trust that the size and pointer values given to us by Assimp are valid.
    */
    unsafe fn from_ai(ai_mesh: *mut AiMesh) -> Mesh {

        let mut vertices: Vec<ModelVertex> = Vec::with_capacity((*ai_mesh).num_vertices as usize);

        for i in 0..(*ai_mesh).num_vertices {
            let vert: *const AiVector3D = (*ai_mesh).vertices.offset(i as isize);

            let mut pos: Vector3f = Vector3f::from_ai(*vert);

            let norm: *const AiVector3D = (*ai_mesh).normals.offset(i as isize);

            let mut normal: Vector3f = Vector3f::from_ai(*norm);

            let mut uv: Vector2f = Vector2f::zero();

            // Check if the this vertex has a texture coordinate.
            if (*ai_mesh).has_texture_coords(0) {
                let ai_uv: *const AiVector3D = (*ai_mesh).texture_coords[0].offset(i as isize);
                uv = Vector2f { x: (*ai_uv).x, y: (*ai_uv).y };
            }

            // Construt and push the ModelVertex.
            vertices.push(ModelVertex::new(pos, normal, uv));
        }
        let mut indices: Vec<u32> = Vec::with_capacity((*ai_mesh).num_faces as usize);

        for i in 0..(*ai_mesh).num_faces {

            let face: *const AiFace = (*ai_mesh).faces.offset(i as isize);
            // This should always be the case as we have triangulated the model.
            assert_eq!((*face).num_indices, 3);

            // Add each of the indices of the faces.
            indices.push(*(*face).indices.offset(0) as u32);
            indices.push(*(*face).indices.offset(1) as u32);
            indices.push(*(*face).indices.offset(2) as u32);

        }

        // This relies on there being the same number of Material objects as there are AiMaterial objects.
        let material_index: usize = (*ai_mesh).material_index as usize;

        let skeleton: Skeleton = Skeleton::from_ai_mesh(ai_mesh, &mut vertices);

        return Mesh { vertices, indices, material_index, skeleton, ai_mesh };

    }

    pub fn get_material<'a>(&self, model: &'a Model) -> &'a MaterialData {

        return &model.materials[self.material_index];

    }

}

pub struct MaterialData {

    pub albedo_path: Option<String>,
    pub normal_path: Option<String>,
    pub metallic_path: Option<String>,
    pub roughness_path: Option<String>,

    pub color: Option<OpaqueColor>,

}

impl MaterialData {

    unsafe fn from_ai(ai_material: *const AiMaterial, path_offset: &std::path::PathBuf) -> Self {

        let mut albedo_path: Option<String> = None;
        let mut normal_path: Option<String> = None;
        let mut metallic_path: Option<String> = None;
        let mut roughness_path: Option<String> = None;

        let mut color: Option<OpaqueColor> = None;

        if aiGetMaterialTextureCount(ai_material, AiTextureType::Diffuse) > 0 {
            let mut path: AiString = AiString::default();
            if aiGetMaterialTexture(ai_material, AiTextureType::Diffuse, 0, &mut path, null(), null_mut(), null_mut(), null_mut(), null_mut(), null_mut()) == AiReturn::Success {
                let mut p = path_offset.clone();
                p.push(path.as_ref());
                albedo_path = Some(p.to_str().expect("STRING ERROR").to_owned());
            }
        }
        if aiGetMaterialTextureCount(ai_material, AiTextureType::Normals) > 0 {
            let mut path: AiString = AiString::default();
            if aiGetMaterialTexture(ai_material, AiTextureType::Normals, 0, &mut path, null(), null_mut(), null_mut(), null_mut(), null_mut(), null_mut()) == AiReturn::Success {
                let mut p = path_offset.clone();
                p.push(path.as_ref());
                normal_path = Some(p.to_str().expect("STRING ERROR").to_owned());
            }
        }
        if aiGetMaterialTextureCount(ai_material, AiTextureType::Shininess) > 0 {
            let mut path: AiString = AiString::default();
            if aiGetMaterialTexture(ai_material, AiTextureType::Shininess, 0, &mut path, null(), null_mut(), null_mut(), null_mut(), null_mut(), null_mut()) == AiReturn::Success {
                let mut p = path_offset.clone();
                p.push(path.as_ref());
                metallic_path = Some(p.to_str().expect("STRING ERROR").to_owned());
            }
        }
        if aiGetMaterialTextureCount(ai_material, AiTextureType::Displacement) > 0 {
            let mut path: AiString = AiString::default();
            if aiGetMaterialTexture(ai_material, AiTextureType::Displacement, 0, &mut path, null(), null_mut(), null_mut(), null_mut(), null_mut(), null_mut()) == AiReturn::Success {
                let mut p = path_offset.clone();
                p.push(path.as_ref());
                roughness_path = Some(p.to_str().expect("STRING ERROR").to_owned());
            }
        }

        return Self { albedo_path, normal_path, metallic_path, roughness_path, color: None };

    }

    pub fn load_material(&self) -> Option<material::Material> {
        let mut albedo_texture: Option<Res<texture::Texture>> = None;
        let mut normal_texture: Option<Res<texture::Texture>> = None;
        let mut metallic_texture: Option<Res<texture::Texture>> = None;
        let mut roughness_texture: Option<Res<texture::Texture>> = None;

        let mut color: OpaqueColor = OpaqueColor::black();

        if let Some(path) = self.albedo_path.as_ref() {
            if let Ok(tex) = texture::Texture::from_file(path) {
                albedo_texture = Some(Res::Heap(Heap::Arc(Arc::new(tex))));
            }
        }
        if let Some(path) = self.normal_path.as_ref() {
            if let Ok(tex) = texture::Texture::from_file(path) {
                normal_texture = Some(Res::Heap(Heap::Arc(Arc::new(tex))));
            }
        }
        if let Some(path) = self.metallic_path.as_ref() {
            if let Ok(tex) = texture::Texture::from_file(path) {
                metallic_texture = Some(Res::Heap(Heap::Arc(Arc::new(tex))));
            }
        }
        if let Some(path) = self.roughness_path.as_ref() {
            if let Ok(tex) = texture::Texture::from_file(path) {
                roughness_texture = Some(Res::Heap(Heap::Arc(Arc::new(tex))));
            }
        }
        if let Some(c) = self.color {
            color = c;
        }
        if albedo_texture.is_none() && normal_texture.is_none() && metallic_texture.is_none() && roughness_texture.is_none() {
            return None;
        }
        return Some(material::Material::new(albedo_texture, normal_texture, metallic_texture, roughness_texture, color, 0.0, 0.0));
    }

}

#[derive(Copy, Clone)]
pub struct VertexWeight {

    pub vertex_id: usize,
    pub weight: f32,

}

impl VertexWeight {

    pub fn new() -> VertexWeight {
        return VertexWeight { vertex_id: 0, weight: 0.0 };
    }

    pub unsafe fn from_ai(ai_weight: *const AiVertexWeight) -> VertexWeight {

        return VertexWeight { vertex_id: (*ai_weight).vertex_id as usize, weight: (*ai_weight).weight };

    }

}

#[derive(Copy, Clone)]
pub struct Bone {

    ai_bone: *mut AiBone,
    pub transform: Matrix4f,
    pub weights: [VertexWeight; 4],
    pub weight_count: usize,

}

impl Bone {

    pub unsafe fn from_ai(ai_bone: *mut AiBone) -> Bone {

        let mut weights: [VertexWeight; 4] = [VertexWeight::new(); 4];

        let mut num: usize = (*ai_bone).num_weights as usize;

        if num > 4 {
            num = 4;
        }

        for i in 0..num {
            weights[i] = VertexWeight::from_ai((*ai_bone).weights.offset(i as isize));
        }

        let transform: Matrix4f = Matrix4f::from_ai((*ai_bone).offset_matrix);

        return Bone { ai_bone: ai_bone, transform: transform, weights, weight_count: num };

    }

    pub fn get_name(&self) -> std::ffi::CString {

        unsafe {

            return std::ffi::CString::new(&(*self.ai_bone).name.data as &[u8]).unwrap();

        }

    }
}

#[derive(Clone)]
pub struct Skeleton {

    pub bones: Vec<Bone>,
    pub transforms: Vec<Matrix4f>,

}

impl Skeleton {

    pub unsafe fn from_ai_mesh(ai_mesh: *mut AiMesh, vertices: &mut Vec<ModelVertex>) -> Skeleton {

        let mut bones: Vec<Bone> = Vec::new();

        for i in 0..(*ai_mesh).num_bones {

            let bone: *mut AiBone = *(*ai_mesh).bones.offset(i as isize);

            for w in 0..(*bone).num_weights {
                let mut i: usize = 0;
                for vert in vertices.iter_mut() {
                    let weight = (*bone).weights.offset(w as isize);
                    if i == (*weight).vertex_id as usize {
                        vert.add_bone((*weight).vertex_id as i32, (*weight).weight);
                    }
                    i += 1;
                }
            }

            bones.push(Bone::from_ai(bone));
        }

        let mut transforms: Vec<Matrix4f> = Vec::with_capacity(bones.len());

        let mut i: usize = 0;
        for b in bones.iter() {

            transforms[i] = b.transform;

            i += 1;
        }

        return Skeleton { bones: bones, transforms: transforms };

    }

    pub fn get_bone_index(&mut self, name: std::ffi::CString) -> isize {

        let mut index: isize = 0;

        for bone in self.bones.iter_mut() {

            if bone.get_name() == name {
                return index;
            }

            index += 1;

        }

        return -1;

    }

}

pub struct Animation {

    ai_anim: *mut AiAnimation,
    pub global_inv_transform: Matrix4f,
    root_node: *mut AiNode,

}

impl Animation {

    pub fn from_ai(ai_anim: *mut AiAnimation, root_node: *mut AiNode, global_inv_transform: Matrix4f) -> Animation {

        return Animation { ai_anim, root_node, global_inv_transform };

    }

    pub fn update_skeleton(&self, skeleton: &mut Skeleton, delta: f64) {

        let identity: Matrix4f = Matrix4f::identity();

        let ticks_per_sec: f64 = unsafe {(*self.ai_anim).ticks_per_second};

        let time_in_ticks = delta * ticks_per_sec;

        let anim_time: f64 = time_in_ticks % unsafe {(*self.ai_anim).duration};

        self.read_node_hierarchy(self.root_node, skeleton, identity, self.global_inv_transform, anim_time);

    }

    fn get_node_anim(&self, name: std::ffi::CString) -> *mut AiNodeAnim {

        unsafe {
            for i in 0..(*self.ai_anim).num_channels {
                let ai_node_anim = (*(*self.ai_anim).channels).offset(i as isize);
                if std::ffi::CString::new(&(*ai_node_anim).node_name.data as &[u8]).unwrap() == name {
                    return ai_node_anim;
                }
            }
        }
        return null_mut();
    }

    fn read_node_hierarchy(&self, node: *mut AiNode, skeleton: &mut Skeleton, parent_transform: Matrix4f, global_transform: Matrix4f, anim_time: f64) {

        unsafe {

            let name: std::ffi::CString = std::ffi::CString::new(&(*node).name.data as &[u8]).unwrap();
            let node_anim: *mut AiNodeAnim = self.get_node_anim(name.clone());

            let mut node_transform: Matrix4f = Matrix4f::from_ai((*node).transformation);

            if node_anim != null_mut() {

                let scaling: Vector3f = Animation::interpolate_scaling(node_anim, anim_time);
                let scaling_matrix: Matrix4f = Matrix4f::from_nonuniform_scale(scaling.x, scaling.y, scaling.z);

                let rotation: Quaternion<f32> = Animation::interpolate_rotation(node_anim, anim_time);
                let rotation_matrix: Matrix4f = Matrix4f::from(rotation);

                let translation: Vector3f = Animation::interpolate_translation(node_anim, anim_time);
                let translation_matrix: Matrix4f = Matrix4f::from_translation(translation);

                node_transform = translation_matrix * rotation_matrix * scaling_matrix;
            }

            let total_transform: Matrix4f = parent_transform * node_transform;

            let bone_index: isize = skeleton.get_bone_index(name.clone());

            if bone_index >= 0 {
                skeleton.transforms[bone_index as usize] = global_transform * total_transform * skeleton.bones[bone_index as usize].transform;
            }

            for i in 0..(*node).num_children {
                self.read_node_hierarchy(*(*node).children.offset(i as isize), skeleton, total_transform, global_transform, anim_time);
            }

        }

    }

    unsafe fn find_node_scaling(node_anim: *mut AiNodeAnim, anim_time: f64) -> usize {

        for i in 0..(*node_anim).num_scaling_keys - 1 {

            if anim_time < (*(*node_anim).scaling_keys.offset((i + 1) as isize)).time as f64 {

                return i as usize;

            }

        }

        return 0;

    }

    unsafe fn find_node_rotation(node_anim: *mut AiNodeAnim, anim_time: f64) -> usize {

        for i in 0..(*node_anim).num_rotation_keys - 1 {

            if anim_time < (*(*node_anim).rotation_keys.offset((i + 1) as isize)).time as f64 {

                return i as usize;

            }

        }

        return 0;

    }

    unsafe fn find_node_position(node_anim: *mut AiNodeAnim, anim_time: f64) -> usize {

        for i in 0..(*node_anim).num_position_keys - 1 {

            if anim_time < (*(*node_anim).position_keys.offset((i + 1) as isize)).time as f64 {

                return i as usize;

            }

        }

        return 0;

    }

    unsafe fn interpolate_translation(node_anim: *mut AiNodeAnim, anim_time: f64) -> Vector3f {

        if (*node_anim).num_position_keys == 1 {

            return Vector3f::from_ai((*(*node_anim).position_keys.offset(0)).value);

        }

        let index: usize = Animation::find_node_position(node_anim, anim_time);

        let next_index: usize = index + 1;

        if next_index >= (*node_anim).num_position_keys as usize {
            panic!("Fatal error when interpolating positions.");
        }

        let delta: f64 = (*(*node_anim).position_keys.offset(index as isize)).time - (*(*node_anim).position_keys.offset(next_index as isize)).time;
        let factor: f64 = (anim_time - (*(*node_anim).position_keys.offset(index as isize)).time) / delta;

        if factor < 0.0 || factor > 1.0 {
            panic!("Invalid factor when interpolating positions");
        }

        let start: Vector3f = Vector3f::from_ai((*(*node_anim).position_keys.offset(index as isize)).value);
        let end: Vector3f = Vector3f::from_ai((*(*node_anim).position_keys.offset(next_index as isize)).value);

        let delta_vec: Vector3f = end - start;

        return start + (delta_vec * factor as f32);

    }

    unsafe fn interpolate_scaling(node_anim: *mut AiNodeAnim, anim_time: f64) -> Vector3f {

        if (*node_anim).num_scaling_keys == 1 {

            return Vector3f::from_ai((*(*node_anim).scaling_keys.offset(0)).value);

        }

        let index: usize = Animation::find_node_scaling(node_anim, anim_time);

        let next_index: usize = index + 1;

        if next_index >= (*node_anim).num_scaling_keys as usize {
            panic!("Fatal error when interpolating positions.");
        }

        let delta: f64 = (*(*node_anim).scaling_keys.offset(index as isize)).time - (*(*node_anim).scaling_keys.offset(next_index as isize)).time;
        let factor: f64 = (anim_time - (*(*node_anim).scaling_keys.offset(index as isize)).time) / delta;

        if factor < 0.0 || factor > 1.0 {
            panic!("Invalid factor when interpolating positions");
        }

        let start: Vector3f = Vector3f::from_ai((*(*node_anim).scaling_keys.offset(index as isize)).value);
        let end: Vector3f = Vector3f::from_ai((*(*node_anim).scaling_keys.offset(next_index as isize)).value);

        let delta_vec: Vector3f = end - start;

        return start + (delta_vec * factor as f32);

    }

    unsafe fn interpolate_rotation(node_anim: *mut AiNodeAnim, anim_time: f64) -> Quaternion<f32> {

        if (*node_anim).num_rotation_keys == 1 {

            let quat: AiQuaternion = (*(*node_anim).rotation_keys.offset(0)).value;

            let start: Quaternion<f32> = Quaternion::new(quat.w, quat.x, quat.y, quat.z);

        }

        let index: usize = Animation::find_node_rotation(node_anim, anim_time);

        let next_index: usize = index + 1;

        if next_index >= (*node_anim).num_rotation_keys as usize {
            panic!("Fatal error when interpolating positions.");
        }

        let delta: f64 = (*(*node_anim).rotation_keys.offset(index as isize)).time - (*(*node_anim).rotation_keys.offset(next_index as isize)).time;
        let factor: f64 = (anim_time - (*(*node_anim).rotation_keys.offset(index as isize)).time) / delta;

        if factor < 0.0 || factor > 1.0 {
            panic!("Invalid factor when interpolating positions");
        }

        let ai_start: AiQuaternion = (*(*node_anim).rotation_keys.offset(index as isize)).value;
        let ai_end: AiQuaternion = (*(*node_anim).rotation_keys.offset(next_index as isize)).value;

        let start: Quaternion<f32> = Quaternion::new(ai_start.w, ai_start.x, ai_start.y, ai_start.z);
        let end: Quaternion<f32> = Quaternion::new(ai_end.w, ai_end.x, ai_end.y, ai_end.z);

        let delta_quat: Quaternion<f32> = end - start;

        return start + (delta_quat * factor as f32);

    }

}

pub struct Model {

    pub meshes: Vec<Mesh>,
    pub materials: Vec<MaterialData>,
    pub animations: Vec<Animation>,
    pub global_inv_transform: Matrix4f,

}

impl Model {

    pub fn new() -> Model {

        return Model { meshes: Vec::new(), materials: Vec::new(), animations: Vec::new(), global_inv_transform: Matrix4f::identity() };

    }

    pub fn from_file(path: &str) -> Result<Model, &'static str> {
        let mut meshes: Vec<Mesh>;

        let mut materials: Vec<MaterialData>;

        let mut animations: Vec<Animation>;

        let mut git: Matrix4f;

        let mut parent_dir: std::path::PathBuf = std::path::PathBuf::from(path);
        parent_dir.pop();

        unsafe {
            let scene: *const AiScene = aiImportFile(std::ffi::CString::new(path).expect("STRING ERROR").as_ptr() as *const i8, AIPROCESS_TRIANGULATE | AIPROCESS_GEN_SMOOTH_NORMALS);

            if scene != null() {
                git = Matrix4f::from_ai((*(*scene).root_node).transformation).inverse_transform().unwrap();
                meshes = Vec::with_capacity((*scene).num_meshes as usize);

                for i in 0..(*scene).num_meshes {
                    meshes.push(Mesh::from_ai(*(*scene).meshes.offset(i as isize)));
                }
                materials = Vec::with_capacity((*scene).num_materials as usize);

                for i in 0..(*scene).num_materials {
                    materials.push(MaterialData::from_ai(*(*scene).materials.offset(i as isize), &parent_dir));
                }
                animations = Vec::with_capacity((*scene).num_animations as usize);

                for i in 0..(*scene).num_animations {
                    animations.push(Animation::from_ai(*(*scene).animations.offset(i as isize), (*scene).root_node, git));
                }
            } else {
                return Err("Failed to load model from path");
            }
        }

        return Ok(Model { meshes, materials, animations, global_inv_transform: git });
    }

    pub fn assign_material(&mut self, mesh_index: usize, material: MaterialData) {
        let mat_index: usize = self.meshes[mesh_index].material_index;
        self.materials[mat_index] = material;
    }

    /// Return the parent node containing all the meshes of the model file.
    pub fn add_to_scene(&self, scene: &mut spatial::Scene3D, graphics: &mut render::Graphics) -> spatial::BaseEntity3D {
        let parent_entity = scene.create_base_entity();
        for mesh in self.meshes.iter() {
            let buffered_mesh: BufferedMesh = BufferedMesh::new(mesh, &graphics.device);
            let material_component: material::MaterialComponent;
            if let Some(material_data) = self.materials.get(mesh.material_index) {
                if let Some(material) = material_data.load_material() {
                    material_component = material::MaterialComponent::new(material, graphics);
                } else {
                    material_component = material::MaterialComponent::new(material::Material::color(OpaqueColor::black(), 0.0, 1.0), graphics);
                }
            } else {
                material_component = material::MaterialComponent::new(material::Material::color(OpaqueColor::black(), 0.0, 1.0), graphics);
            }
            let entity = scene.basic_builder().with(buffered_mesh).with(material_component).with(scene::Parent::new(parent_entity.entity)).build();
        }
        return parent_entity;
    }

}

