use nalgebra_glm as glm;

use crate::resources::{
    Resources,
    Mesh, 
    Material, 
    Shader
};

// Object -> position, rotation, scale, mesh, material
pub struct Object {
    position: glm::Vec3,
    rotation: glm::Quat,
    scale: glm::Vec3,

    model: glm::Mat4,

    mesh_id: String,
    material_id: String,
    
    children: Vec<Object>,
}

impl Object {
    pub fn new(position: glm::Vec3, rotation: glm::Quat, scale: glm::Vec3, mesh_id: &str, material_id: &str) -> Self {
        Self {
            position,
            rotation,
            scale,
            mesh_id: mesh_id.to_string(),
            material_id: material_id.to_string(),
            model: glm::Mat4::identity(),
            children: Vec::new(),
        }
    }

    pub fn update_model_matrix(&mut self) {
        // The order (Translate * Rotate * Scale) is for the local coordinate system
        // In matrix multiplication, this means: M = T * R * S
        // glm operations apply transformations from right to left if you multiply
        // the matrix by the new transformation: m_model = transform * m_model
        // but if you do m_model = m_model * transform, it applies in world space
        // The way done above is more intuitive and standard: build T, then multiply by R, then by S.
        self.model = glm::Mat4::identity();
        self.model = glm::translate(&mut self.model, &self.position);
        self.model = self.model * glm::quat_cast(&self.rotation);
        self.model = glm::scale(&mut self.model, &self.scale);
    }

    pub fn draw(
        &self, 
        view: &glm::Mat4, 
        projection: &glm::Mat4,
        gl: &glow::Context,
        resources: &Resources,
        as_wireframe: bool,
    ) {
        let material = resources.get_material(&self.material_id).expect("Material not found");
        
        
        if let Some(shader_name) = &material.shader {
            let shader = resources.get_shader(shader_name).expect("Shader not found");
            shader.use_program(gl);
            
            // 3. Set uniforms
            shader.set_mat4(gl, "model", &self.model);
            shader.set_mat4(gl, "view", view);
            shader.set_mat4(gl, "projection", projection);

            // 4. Draw the actual mesh
            let mesh = resources.get_mesh(&self.mesh_id).expect("Mesh not found");
            mesh.draw(gl, as_wireframe);
        }
    }
}

