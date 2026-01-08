pub mod window;
pub mod camera;
pub mod resources;
pub mod shaders;
pub mod scenes;
pub mod object;

use anyhow::Result;
use glow::HasContext;
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::event::{
    ElementState, 
    KeyEvent, 
    WindowEvent, 
    MouseScrollDelta, 
    MouseButton
};
use winit::event_loop::{
    ActiveEventLoop, 
    ControlFlow, 
    EventLoop
};
use winit::keyboard::{
    KeyCode, 
    PhysicalKey, 
    Key, 
    KeyLocation
};
use winit::window::CursorGrabMode;

pub use crate::resources::Resources;
pub use crate::window::Window;
pub use crate::camera::Camera;
pub use crate::scenes::Scene;

// the main entrypoint for the CAD-3E Engine library
// is this struct which just encapsulates the whole
// - window,
// - camera, 
// - resources (shaders, textures, meshes, materials),
// - components
// - scenes (Objects, Lights, Cameras, etc...)
// it's those structures where the real data and logic will be implemented
// the Engine just provides the framework to run everything

// 3D graphics itself is not very complex in itself
// the core idea is to calculate a 3D position in space
// and then project it to a 2D surface (aka your screen)
// let a point P(x, y, z) be a point in 3D space
// let P' (x', y',z') be the projected point on the 2D surface
// assuming the camera is at the origin (0,0,0) looking down the -Z axis (so Z is depth, how  far something is from the camera/eye/viewer)
// then we can create a triangle between the point P, the origin (camera) and the projection plane (the screen)
// using similar triangles we can derive the following equations:
// x' = (x / z)
// y' = (y / z)
// this is a simplified version of the perspective projection
// in real applications we use projection matrices to handle this transformation
// which also take into account: 
// - the field of view (FOV) 
//   -> (perspective or orthographic projection) 
// - aspect ratio 
//   -> (width / height)
// - near and far clipping planes 
//   -> (how close or far objects can be to be rendered)
// - depth buffer (Z-buffer) to handle occlusion 
//   -> (one object in front of another)
// - viewport transformation to map normalized device coordinates to screen coordinates 
//   -> (from coordinates between -1 and 1 to actual pixel coordinates on the screen)
// https://learnopengl.com/Getting-started/Coordinate-Systems
// by taking into account all these factors into account we can create a illusion of depth and 3D space on a 2D screen
// that is the core idea behind 3D graphics rendering

// so in theory you can make a 3D engine using just a software rasterizer
// writing pixels directly to a framebuffer in memory
// however this is very inefficient and slow when more points need to be drawn and 
// not practical for real-time applications
// that is where graphics APIs (like OpenGL, DirectX, Vulkan, Metal, etc) come into play
// a graphics API allows us to leverage the power of the GPU (Graphics Processing Unit)
// which is specialized hardware designed to handle parallel processing of graphics data
// using a graphics API we can offload the heavy lifting of rendering to the GPU
// allowing for real-time rendering of complex 3D scenes up to millions of polygons
// modern graphics APIs also provide advanced features like shaders
// which are small programs that run on the GPU to control the rendering pipeline
// allowing for even more calculations to be applied to a point 
// to apply effects like lighting, shadows, textures, post-processing, etc... 
// 
// if we further extend these concepts and don't just simply use points in 3D space
// to render something we can use collections of points called meshes
// a mesh is a collection of (points in 3D space)
// extending this further we can group meshes into objects
// each object can have its own position, rotation, scale in the 3D world
// so the "object" is a higher level abstraction that contains one or more meshes
// but spatial information is not the only data we can give to an object
// we can also give it materials (which define how the object looks when rendered)
// this can go from simple colors to complex shaders with multiple texture maps
// combining these more cobinations of data we no longer just have points but vertices
// a vertex is piece of data:
// - position in 3D space (x, y, z, w)
// - normals (for lighting calculations), 
// - texture coordinates (for mapping textures), 
// - colors, 
// - etc...

// schema:
// Scene -> [Objects] + Camera(your viewport or external view) + [Lights]
// Object -> [Mesh1, Mesh2, ...] + Transform + Material
// Mesh -> [Vertex1, Vertex2, ...](VBO) + Indices(EBO)
// Vertex -> Position + Normal + TexCoords + Color + ... (as seemed fit by the engine/user)

// each vertex positional data is then transformed by the object's transform (position, rotation, scale)
// then by the camera's view matrix (to convert world space to view/camera space)
// then by the projection matrix (to convert view space to clip space)
// finally the projected 2D coordinates are mapped to screen space for rendering
// this series of transformations allows us to render 3D objects onto a 2D screen

// 3 Major matrices should be noted (the MVP's :) ):
// 1) Model Matrix		-> This matrix transforms vertices from a model/mesh's local space to world space
// 2) View Matrix		-> This matrix represents the camera's position and orientation transforming vertices from world space into the camera's view space
// 3) Projection Matrix -> This matrix defines how the 3D scene is projected onto a 2D screen (e.g.: perspective or orthographic) transforming view space into clip space (because well... it clips uncessary objects that don't need to be rendered)

// then we just have to modify these matrices once per object and once per frame 
// and in our vertex shader we multiply these matrices => projection * view * model * vertexPosition
// https://learnopengl.com/img/getting-started/coordinate_systems.png
// http://www.songho.ca/opengl/gl_projectionmatrix.html


pub struct CAD3EEngine {
    delta_time: f32,
    last_frame: Instant,
    target_fps: u32,
    
    gl: Option<glow::Context>,
    window: Window,
    resources: Resources,
    scenes: Vec<Scene>,

    // input states of each key and mouse button
    keys: [bool; 1024],
    last_mouse_pos: (f64, f64),
    is_dragging: bool,

    initialized: bool,
    
    enable_depth_test: bool,
    enable_v_sync: bool,
    enable_capture_cursor: bool,
    enable_wireframe_mode: bool,
    enable_backface_culling: bool,
    enable_msaa: bool,
}


impl CAD3EEngine {
    pub fn new(width: u32, height: u32, title: &str, target_fps: u32) -> Self {
        if target_fps == 0 {
            panic!("FPS cannot be zero");
        }

        Self {
            gl: None,
            window: Window::new(width, height, title),
            resources: Resources::new(),
            scenes: Vec::new(),

            delta_time: 0.0,
            last_frame: Instant::now(),
            target_fps,
            keys: [false; 1024],
            last_mouse_pos: (0.0, 0.0),
            is_dragging: false,
            initialized: false,
            enable_depth_test: true,
            enable_v_sync: true,
            enable_capture_cursor: false,
            enable_wireframe_mode: false,
            enable_backface_culling: false,
            enable_msaa: true,
        }
    }

    // starts the engine main loop for event handling 
    // that will be passed to winit's event loop
    pub fn run(mut self) -> Result<()> {

        let event_loop = EventLoop::new()
            .map_err(|e| anyhow::anyhow!("Failed to create event loop: {}", e))?;

        event_loop
            .set_control_flow(ControlFlow::Poll);
        
        event_loop
            .run_app(&mut self)
            .map_err(|e| anyhow::anyhow!("Event loop error: {}", e))?;
        
        Ok(())
    }
    
    fn initialize(&mut self, event_loop: &ActiveEventLoop) -> Result<()> {

        // we set initialized to true at the top 
        // this prevents re-initialization if already done
        // even when an error occurs during initialization
        // a tip from Freya Holmér
        self.initialized = true;
        // first the window is initialized which:
        // 1) creates the window
        // 2) sets up event handling for the window
        // 3) sets up a OpenGL context 
        let gl = self.window.init(event_loop)?;
        unsafe {
            if self.enable_depth_test {
                // enables OpenGL to use the Z-buffer (in c++ this is glEnable(GL_DEPTH_TEST);)
                gl.enable(glow::DEPTH_TEST);
                gl.depth_func(glow::LESS);
            }
            
            if self.enable_backface_culling {
                gl.enable(glow::CULL_FACE);
                gl.cull_face(glow::BACK);
            }

            if self.enable_msaa {
                // MSAA (multisample anti-aliasing)
                gl.enable(glow::MULTISAMPLE);
            }

            if self.enable_wireframe_mode {
                gl.polygon_mode(glow::FRONT_AND_BACK, glow::LINE);
            } else {
                gl.polygon_mode(glow::FRONT_AND_BACK, glow::FILL);
            }

            // set the viewport to the window size in which OpenGL will render
            gl.viewport(0, 0, self.window.width() as i32, self.window.height() as i32);
        }
        // finally we store the GL context for later use
        self.gl = Some(gl);

        Ok(())
    }

    pub fn clear(&self, r: f32, g: f32, b: f32, a: f32) {
        // when we clear the screen we also clear the depth buffer
        // later on it might be useful to clear stencil buffer as well
        if let Some(gl) = &self.gl {
            unsafe {
                gl.clear_color(r, g, b, a);
                gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
            }
        }
    }

    pub fn update(&mut self) {
        let current_frame = Instant::now();
        self.delta_time = current_frame.duration_since(self.last_frame).as_secs_f32();
        self.last_frame = current_frame;
    }

    pub fn render(&self) {
        // for rendering we just iterate over all scenes
        // and call their render method (pass also the GL context)
        for scene in &self.scenes {
            scene.render(
                self.gl.as_ref().unwrap(),
                &self.resources,
            );
        }

        self.window
            .swap_buffers()
            .unwrap_or_else(|e| eprintln!("Failed to swap buffers: {}", e));
    }

    pub fn gl(&self) -> Option<&glow::Context> {
        self.gl.as_ref()
    }

    pub fn resources(&self) -> &Resources {
        &self.resources
    }

    pub fn resources_mut(&mut self) -> &mut Resources {
        &mut self.resources
    }

    pub fn delta_time(&self) -> f32 {
        self.delta_time
    }

    pub fn window(&self) -> &Window {
        &self.window
    }
}

// https://medium.com/@ethan_38158/lets-build-a-graphics-engine-in-rust-the-humble-triangle-8a207320afe2
// https://docs.rs/winit/latest/winit/#event-handling
impl ApplicationHandler for CAD3EEngine {

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if !self.initialized {
            if let Err(e) = self.initialize(event_loop) {
                eprintln!("Failed to initialize engine: {}", e);
                event_loop.exit();
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: winit::window::WindowId,
        window_event: WindowEvent,
    ) {
        match window_event {
            WindowEvent::CloseRequested => {
                println!("Close requested, exiting...");
                event_loop.exit();
            },
            WindowEvent::Resized(physical_size) => {
                self.window.resize(physical_size.width, physical_size.height);

                if let Some(gl) = &self.gl {
                    unsafe {
                        gl.viewport(
                            0,
                            0,
                            physical_size.width as i32,
                            physical_size.height as i32,
                        );
                    }
                }
                // for each scene we might want to update the camera's projection matrix
                for scene in &mut self.scenes {
                    scene.get_camera_mut().update_projection(
                        physical_size.width as f32,
                        physical_size.height as f32,
                    );
                }

            },
            WindowEvent::RedrawRequested => {
                self.update();
                self.clear(0.1, 0.1, 0.15, 1.0);             
                self.render();
                self.window.request_redraw();
            },
            // https://github.com/rust-windowing/winit/blob/master/winit/examples/control_flow.rs
            WindowEvent::KeyboardInput { event, .. } => {

                if let PhysicalKey::Code(key_code) = event.physical_key {
                    let pressed = event.state == ElementState::Pressed;
                    match key_code  {
                        KeyCode::Escape if pressed => {
                            println!("Escape pressed, exiting...");
                            event_loop.exit();
                        },
                        KeyCode::Digit1 if pressed => {
                            self.enable_depth_test = !self.enable_depth_test;
                            
                            if let Some(gl) = &self.gl {
                                if self.enable_depth_test {
                                    unsafe {
                                        gl.enable(glow::DEPTH_TEST);
                                    }
                                } else {
                                    unsafe {
                                        gl.disable(glow::DEPTH_TEST);
                                    }
                                }
                            }
                            println!("Toggled depth test: {}", self.enable_depth_test);
                        },
                        KeyCode::Digit2 if pressed => {
                            self.enable_capture_cursor = !self.enable_capture_cursor;
                            if self.enable_capture_cursor {
                                self.window.winit_window()
                                    .as_ref()
                                    .unwrap()
                                    .set_cursor_grab(CursorGrabMode::Confined)
                                    .unwrap_or_else(|e| eprintln!("Failed to capture cursor: {}", e));
                            } else {
                                self.window.winit_window()
                                    .as_ref()
                                    .unwrap()
                                    .set_cursor_grab(CursorGrabMode::None)
                                    .unwrap_or_else(|e| eprintln!("Failed to release cursor: {}", e));
                            }
                            println!("Toggled cursor capture: {}", self.enable_capture_cursor);
                        },
                        _ => {
                            let index = key_code as usize;
                            if index < self.keys.len() {
                                self.keys[index] = pressed;
                            }
                        },
                    }
                }

            },
            WindowEvent::CursorMoved { position, .. } => {
                if self.is_dragging {
                    let x_offset = position.x - self.last_mouse_pos.0;
                    let y_offset = self.last_mouse_pos.1 - position.y; // Inverted Y

                    if self.keys[KeyCode::ShiftLeft as usize] && self.is_dragging {
                        self.scenes[0].get_camera_mut().process_pan(x_offset as f32, y_offset as f32);
                    } else {
                        self.scenes[0].get_camera_mut().process_orbit(x_offset as f32, y_offset as f32);
                    }
                }
                self.last_mouse_pos = (position.x, position.y);
            },
            WindowEvent::MouseInput { state, button, .. } => {
                match button {
                    MouseButton::Left => {
                        self.is_dragging = state == ElementState::Pressed;
                    },
                    _ => {},
                }
            },
            WindowEvent::MouseWheel { delta, .. } => {
                if let MouseScrollDelta::LineDelta(_, y) = delta {
                    self.scenes[0].get_camera_mut().process_zoom(y);
                }
            },
            _ => {
                println!("Unhandled window event: {:?}", window_event);
            }
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if self.initialized {
            self.window.request_redraw();
        }
    }
    
}

impl Drop for CAD3EEngine {
    fn drop(&mut self) {
        if let Some(gl) = &self.gl {
            self.resources.cleanup(gl);
        }
    }
}