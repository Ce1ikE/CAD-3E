# CAD-3E Engine

A modern 3D rendering engine written in Rust using OpenGL.

## Architecture

The engine follows a component-based architecture with clear separation of concerns:

### Core Components

#### 1. **CAD3EEngine** (Main Engine Class)
The main engine class that encapsulates all engine logic and orchestrates the rendering pipeline.

**Location**: `CAD-3E-Engine/src/lib.rs`

**Features**:
- Event loop management via `winit`
- Frame timing and delta time calculation
- Window lifecycle management
- OpenGL context initialization
- Integrates all subsystems (Window, Camera, Resources)

**Key Methods**:
- `new(width, height, title, fps)` - Creates a new engine instance
- `run()` - Starts the main event loop
- `clear(r, g, b, a)` - Clears the screen with a color
- `update()` - Updates timing and frame delta
- `render()` - Swaps buffers and presents the frame

#### 2. **Window** 
Manages the OS window and OpenGL context using `glutin` and `winit`.

**Location**: `CAD-3E-Engine/src/window.rs`

**Features**:
- Cross-platform window creation
- OpenGL context creation and management
- Window resizing and aspect ratio handling
- Surface management for rendering

**Key Methods**:
- `new(width, height, title)` - Creates window configuration
- `init(event_loop)` - Initializes window and OpenGL context
- `swap_buffers()` - Presents the rendered frame
- `resize(width, height)` - Handles window resizing

#### 3. **Camera**
3D camera with view and projection matrix management.

**Location**: `CAD-3E-Engine/src/camera.rs`

**Features**:
- First-person camera controls
- Perspective projection
- Mouse and keyboard input handling
- Euler angle rotation (yaw, pitch)
- Configurable movement speed and sensitivity

**Key Methods**:
- `new(position, up, yaw, pitch, aspect_ratio)` - Creates a camera
- `default(aspect_ratio)` - Creates camera at default position
- `get_view_matrix()` - Returns view transformation matrix
- `get_projection_matrix()` - Returns projection matrix
- `process_keyboard(direction, delta_time)` - Handles movement
- `process_mouse_movement(xoffset, yoffset)` - Handles rotation
- `process_mouse_scroll(yoffset)` - Handles zoom

#### 4. **ResourceManager**
Centralized manager for all engine resources (meshes, shaders, textures, materials).

**Location**: `CAD-3E-Engine/src/resources.rs`

**Features**:
- ID-based resource management
- Automatic resource cleanup
- Type-safe resource handles

**Sub-components**:

##### **Mesh**
Represents 3D geometry with vertices and indices.
- Vertex data: position, normal, texture coordinates
- Automatic VAO/VBO/EBO creation
- Efficient GPU buffer management

##### **Shader**
OpenGL shader program with vertex and fragment shaders.
- Shader compilation and linking
- Uniform variable management
- Error reporting for shader compilation

##### **Texture**
2D textures for materials.
- Image loading from files (via `image` crate)
- Mipmap generation
- Texture parameter configuration

##### **Material**
Defines surface properties.
- Albedo color
- Metallic/Roughness (PBR ready)
- Ambient occlusion
- Texture references

## Dependencies

```toml
glutin = "0.32"           # OpenGL context creation
glutin-winit = "0.5"      # Integration layer
glow = "0.16"             # OpenGL bindings
winit = "0.30"            # Cross-platform windowing
nalgebra-glm = "0.18"     # Linear algebra and math
image = "0.25"            # Image loading
anyhow = "1.0"            # Error handling
```

## Usage Example

### Basic Window

```rust
use cad_3e_engine::CAD3EEngine;

fn main() {
    let engine = CAD3EEngine::new(800, 600, "My 3D App", 60);
    engine.run().unwrap();
}
```

### Advanced Usage with 3D Objects

```rust
use cad_3e_engine::*;

fn main() {
    let mut engine = CAD3EEngine::new(1280, 720, "3D Scene", 60);
    
    // After initialization, in your render loop:
    // 1. Load shaders
    let shader_id = engine.resources_mut().add_shader(
        engine.gl().unwrap(),
        VERTEX_SHADER_SOURCE,
        FRAGMENT_SHADER_SOURCE
    ).unwrap();
    
    // 2. Create mesh
    let mesh_id = engine.resources_mut().add_mesh(
        engine.gl().unwrap(),
        vertices,
        indices
    ).unwrap();
    
    // 3. Create material
    let material = Material::new([1.0, 0.5, 0.3]);
    let material_id = engine.resources_mut().add_material(material);
    
    // 4. Render
    let shader = engine.resources().get_shader(shader_id).unwrap();
    shader.use_program(engine.gl().unwrap());
    
    let view = engine.camera().unwrap().get_view_matrix();
    let projection = engine.camera().unwrap().get_projection_matrix();
    shader.set_mat4(engine.gl().unwrap(), "view", &view);
    shader.set_mat4(engine.gl().unwrap(), "projection", &projection);
    
    let mesh = engine.resources().get_mesh(mesh_id).unwrap();
    mesh.draw(engine.gl().unwrap());
}
```

## Building

```bash
cargo build --release
```

## Running

```bash
# Run the basic editor
cargo run

# Run the cube demo example
cargo run --example cube_demo
```

## Architecture Benefits

1. **Encapsulation**: Engine class contains all subsystems
2. **Resource Management**: Centralized, ID-based resource tracking
3. **Type Safety**: Rust's type system prevents common graphics bugs
4. **Modern Graphics**: Uses modern OpenGL (3.3+) with programmable pipeline
5. **Cross-platform**: Works on Windows, macOS, and Linux via `winit`
6. **Memory Safe**: Automatic cleanup with Drop trait implementation

## Future Enhancements

- [ ] Scene graph for hierarchical transformations
- [ ] PBR (Physically Based Rendering) material system
- [ ] Shadow mapping
- [ ] Post-processing effects
- [ ] Model loading (GLTF, OBJ)
- [ ] Entity Component System (ECS)
- [ ] Multi-threaded rendering
- [ ] Deferred rendering pipeline
- [ ] UI overlay system

## License

MIT
