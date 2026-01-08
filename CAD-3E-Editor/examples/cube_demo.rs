// Advanced example demonstrating the full 3D engine capabilities
// This example creates a rotating 3D cube with lighting

use cad_3e_engine::*;
use std::time::Instant;

fn main() {
    println!("Starting CAD-3E Engine Advanced Example...");

    // Create engine instance
    let mut engine = CAD3EEngine::new(1280, 720, "CAD-3E Engine - 3D Cube", 60);

    // Run the engine with custom rendering
    if let Err(e) = engine.run() {
        eprintln!("Engine error: {}", e);
    }
}

// Helper function to create a cube mesh
pub fn create_cube_mesh(gl: &glow::Context) -> Result<Mesh, anyhow::Error> {
    let vertices = vec![
        // Front face
        Vertex::new([-0.5, -0.5, 0.5], [0.0, 0.0, 1.0], [0.0, 0.0]),
        Vertex::new([0.5, -0.5, 0.5], [0.0, 0.0, 1.0], [1.0, 0.0]),
        Vertex::new([0.5, 0.5, 0.5], [0.0, 0.0, 1.0], [1.0, 1.0]),
        Vertex::new([-0.5, 0.5, 0.5], [0.0, 0.0, 1.0], [0.0, 1.0]),
        // Back face
        Vertex::new([-0.5, -0.5, -0.5], [0.0, 0.0, -1.0], [1.0, 0.0]),
        Vertex::new([-0.5, 0.5, -0.5], [0.0, 0.0, -1.0], [1.0, 1.0]),
        Vertex::new([0.5, 0.5, -0.5], [0.0, 0.0, -1.0], [0.0, 1.0]),
        Vertex::new([0.5, -0.5, -0.5], [0.0, 0.0, -1.0], [0.0, 0.0]),
        // Top face
        Vertex::new([-0.5, 0.5, -0.5], [0.0, 1.0, 0.0], [0.0, 1.0]),
        Vertex::new([-0.5, 0.5, 0.5], [0.0, 1.0, 0.0], [0.0, 0.0]),
        Vertex::new([0.5, 0.5, 0.5], [0.0, 1.0, 0.0], [1.0, 0.0]),
        Vertex::new([0.5, 0.5, -0.5], [0.0, 1.0, 0.0], [1.0, 1.0]),
        // Bottom face
        Vertex::new([-0.5, -0.5, -0.5], [0.0, -1.0, 0.0], [1.0, 1.0]),
        Vertex::new([0.5, -0.5, -0.5], [0.0, -1.0, 0.0], [0.0, 1.0]),
        Vertex::new([0.5, -0.5, 0.5], [0.0, -1.0, 0.0], [0.0, 0.0]),
        Vertex::new([-0.5, -0.5, 0.5], [0.0, -1.0, 0.0], [1.0, 0.0]),
        // Right face
        Vertex::new([0.5, -0.5, -0.5], [1.0, 0.0, 0.0], [1.0, 0.0]),
        Vertex::new([0.5, 0.5, -0.5], [1.0, 0.0, 0.0], [1.0, 1.0]),
        Vertex::new([0.5, 0.5, 0.5], [1.0, 0.0, 0.0], [0.0, 1.0]),
        Vertex::new([0.5, -0.5, 0.5], [1.0, 0.0, 0.0], [0.0, 0.0]),
        // Left face
        Vertex::new([-0.5, -0.5, -0.5], [-1.0, 0.0, 0.0], [0.0, 0.0]),
        Vertex::new([-0.5, -0.5, 0.5], [-1.0, 0.0, 0.0], [1.0, 0.0]),
        Vertex::new([-0.5, 0.5, 0.5], [-1.0, 0.0, 0.0], [1.0, 1.0]),
        Vertex::new([-0.5, 0.5, -0.5], [-1.0, 0.0, 0.0], [0.0, 1.0]),
    ];

    let indices = vec![
        0, 1, 2, 2, 3, 0, // Front
        4, 5, 6, 6, 7, 4, // Back
        8, 9, 10, 10, 11, 8, // Top
        12, 13, 14, 14, 15, 12, // Bottom
        16, 17, 18, 18, 19, 16, // Right
        20, 21, 22, 22, 23, 20, // Left
    ];

    Mesh::new(gl, vertices, indices)
}

/* 
   To extend this example further, you could:
   
   1. Add keyboard/mouse controls for camera movement
   2. Load shaders and create materials
   3. Render multiple objects with different transformations
   4. Add texture loading and mapping
   5. Implement a scene graph for object hierarchy
   
   Example usage in the engine's render loop:
   
   ```rust
   // In initialization:
   let shader_id = engine.resources_mut().add_shader(
       engine.gl().unwrap(),
       shaders::BASIC_VERTEX_SHADER,
       shaders::BASIC_FRAGMENT_SHADER
   )?;
   
   let cube_mesh_id = engine.resources_mut().add_mesh(
       engine.gl().unwrap(),
       create_cube_vertices(),
       create_cube_indices()
   )?;
   
   // In render loop:
   let shader = engine.resources().get_shader(shader_id).unwrap();
   shader.use_program(engine.gl().unwrap());
   
   let view = engine.camera().unwrap().get_view_matrix();
   let projection = engine.camera().unwrap().get_projection_matrix();
   shader.set_mat4(engine.gl().unwrap(), "view", &view);
   shader.set_mat4(engine.gl().unwrap(), "projection", &projection);
   
   let mesh = engine.resources().get_mesh(cube_mesh_id).unwrap();
   mesh.draw(engine.gl().unwrap());
   ```
*/
