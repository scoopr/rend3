use crate::{
    resources::MeshManager,
    types::{MaterialHandle, Object, ObjectHandle},
    util::{frustum::BoundingSphere, registry::ResourceRegistry},
};
use glam::{Mat4, Vec3A};
use rend3_types::RawObjectHandle;

/// Internal representation of a Object.
#[derive(Debug, Clone)]
pub struct InternalObject {
    pub material: MaterialHandle,
    pub transform: Mat4,
    pub sphere: BoundingSphere,
    pub location: Vec3A,
    pub start_idx: u32,
    pub count: u32,
    pub vertex_offset: i32,
}

/// Manages objects. That's it. ¯\\\_(ツ)\_/¯
pub struct ObjectManager {
    registry: ResourceRegistry<InternalObject, Object>,
}
impl ObjectManager {
    pub fn new() -> Self {
        let registry = ResourceRegistry::new();

        Self { registry }
    }

    pub fn allocate(&self) -> ObjectHandle {
        self.registry.allocate()
    }

    pub fn fill(&mut self, handle: &ObjectHandle, object: Object, mesh_manager: &MeshManager) {
        let mesh = mesh_manager.internal_data(object.mesh.get_raw());

        let shader_object = InternalObject {
            material: object.material,
            transform: object.transform,
            sphere: mesh.bounding_sphere,
            location: object.transform.transform_point3a(Vec3A::ZERO),
            start_idx: mesh.index_range.start as u32,
            count: (mesh.index_range.end - mesh.index_range.start) as u32,
            vertex_offset: mesh.vertex_range.start as i32,
        };

        self.registry.insert(handle, shader_object);
    }

    pub fn ready(&mut self) -> Vec<InternalObject> {
        profiling::scope!("Object Manager Ready");
        self.registry.remove_all_dead(|_, _, _| ());
        self.registry.values().cloned().collect()
    }

    pub fn set_object_transform(&mut self, handle: RawObjectHandle, transform: Mat4) {
        let object = self.registry.get_mut(handle);
        object.transform = transform;
        object.location = transform.transform_point3a(Vec3A::ZERO)
    }
}

impl Default for ObjectManager {
    fn default() -> Self {
        Self::new()
    }
}
