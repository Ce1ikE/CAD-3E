
use glow::{
    Buffer, 
    Framebuffer, 
    HasContext, 
    PixelUnpackData, 
    Texture
};

use image::Pixel;

use crate::camera::Camera;
use crate::object::Object;
use crate::resources::{self, Shader};


// Scene -> a collection of objects, lights, cameras, etc.
pub struct Scene {
    // some people use a scene graph to organize the objects in the scene
    // a scene graph is a hierarchical structure where each node represents an object
    // therfore when a object is added to the scene we add a reference to it's parent object
    objects: Vec<Object>,
    camera: Camera,
    g_buffer: Option<Framebuffer>,
    g_buffer_shader: Option<Shader>,
    lighting_pass_shader: Option<Shader>,
    as_wireframe: bool,
}

// https://rtarun9.github.io/blogs/deferred_shading/
// https://learnopengl.com/Advanced-Lighting/Deferred-Shading
// since screen resolution have been increasing a lot over the years
// and with that the amount of pixels that need to be rendered per frame
// the fragment shader has become the bottleneck in the rendering pipeline
// things like DLSS (deep learning super sampling) are designed to reduce the load on the fragment shader
// by rendering at a lower resolution and then upscaling the image using AI techniques
// however a more simple approach to reduce the load on the fragment shader
// is called Deferred Shading
// which basically consists of splitting the rendering process into two main passes
// the first pass is called the Geometry Pass (G Pass)
// in which we render all the geometry of the scene
// and store all the necessary information (like position, normal, albedo, metallic, roughness, etc...) 
// into multiple render targets (MRTs) (also called G-buffers)
// these MRTs are basically textures that store the output of the fragment shader for each pixel
// in the second pass called the Lighting Pass
// we use the information stored in the MRTs to calculate the final color of each pixel
// this way we can reduce the number of times we need to run the fragment shader
// since we only need to run it once per pixel in the Geometry Pass
// therfore in the Scene struct when the engine iterates over all the scenes to render them
// (normally there is only one scene active at a time)
// it's in the scene struct where we would implement the logic for deferred shading
// each object has it's own material and mesh
// so we need to pick the right shader for each object during the Geometry Pass
// and then in the Lighting Pass we can use a single shader to calculate the final color of

impl Scene {
    pub fn new(camera: Camera) -> Self {
        Self {
            objects: Vec::new(),
            camera,
            as_wireframe: false,
            g_buffer: None,
            g_buffer_shader: None,
            lighting_pass_shader: None,
        }
    }

    pub fn toggle_wireframe(&mut self) {
        self.as_wireframe = !self.as_wireframe;
    }

    pub fn initialize_g_buffer(&mut self, gl: &glow::Context, width: i32, height: i32) {
        // create the G-buffer framebuffer and its associated textures (MRTs)
        // position, normal, albedo + specular
        // depth buffer
        // bind the framebuffer and attach the textures to it
        // unbind the framebuffer
        // https://learnopengl.com/Advanced-Lighting/Deferred-Shading
        unsafe {
            let g_buffer = gl.create_framebuffer().expect("Failed to create G-buffer framebuffer");
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(g_buffer));
            // create textures for position, normal, albedo + specular
            let g_position = gl.create_texture().expect("Failed to create G-buffer position texture");
            gl.bind_texture(glow::TEXTURE_2D, Some(g_position));
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA16F as i32,
                width,
                height,
                0,
                glow::RGBA,
                glow::FLOAT,
                PixelUnpackData::Slice(None),
            );

            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::NEAREST as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::NEAREST as i32);
            gl.framebuffer_texture_2d(
                glow::FRAMEBUFFER,
                glow::COLOR_ATTACHMENT0,
                glow::TEXTURE_2D,
                Some(g_position),
                0,
            );

            let g_normal = gl.create_texture().expect("Failed to create G-buffer normal texture");
            gl.bind_texture(glow::TEXTURE_2D, Some(g_normal));
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA16F as i32,
                width,
                height,
                0,
                glow::RGBA,
                glow::FLOAT,
                PixelUnpackData::Slice(None),
            );
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::NEAREST as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::NEAREST as i32);
            gl.framebuffer_texture_2d(
                glow::FRAMEBUFFER,
                glow::COLOR_ATTACHMENT1,
                glow::TEXTURE_2D,
                Some(g_normal),
                0,
            );

            let g_albedo_spec = gl.create_texture().expect("Failed to create G-buffer albedo + specular texture");
            gl.bind_texture(glow::TEXTURE_2D, Some(g_albedo_spec));
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA as i32,
                width,
                height,
                0,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                PixelUnpackData::Slice(None),
            );
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::NEAREST as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::NEAREST as i32);
            gl.framebuffer_texture_2d(
                glow::FRAMEBUFFER,
                glow::COLOR_ATTACHMENT2,
                glow::TEXTURE_2D,
                Some(g_albedo_spec),
                0,
            );

            // tell OpenGL which color attachments we'll use (of this framebuffer) for rendering 
            let attachments = [
                glow::COLOR_ATTACHMENT0, 
                glow::COLOR_ATTACHMENT1, 
                glow::COLOR_ATTACHMENT2
            ];
            gl.draw_buffers(&attachments);
            // create and attach depth buffer (renderbuffer)
            let rbo_depth = gl.create_renderbuffer().expect("Failed to create G-buffer depth renderbuffer");
            gl.bind_renderbuffer(glow::RENDERBUFFER, Some(rbo_depth));
            gl.renderbuffer_storage(
                glow::RENDERBUFFER,
                glow::DEPTH_COMPONENT,
                width,
                height,
            );
            gl.framebuffer_renderbuffer(
                glow::FRAMEBUFFER,
                glow::DEPTH_ATTACHMENT,
                glow::RENDERBUFFER,
                Some(rbo_depth),
            );
            // finally check if framebuffer is complete
            if gl.check_framebuffer_status(glow::FRAMEBUFFER) != glow::FRAMEBUFFER_COMPLETE {
                panic!("G-buffer Framebuffer is not complete!");
            }
            gl.bind_framebuffer(glow::FRAMEBUFFER, None);
            self.g_buffer = Some(g_buffer);
        }
    
    
    }

    pub fn add_object(&mut self, object: Object) {
        self.objects.push(object);
    }

    pub fn render(&self, gl: &glow::Context, resources: &resources::Resources) {
        // NOTE: the G-bufer can be quite large depending on the screen resolution and number of MRTs
        // therfore it's a good idea to create it once during the initialization of the engine
        // and sometimes just a normal forward rendering pass is less expensive than a deferred shading pass
        // also deferred shading has some limitations like handling transparency and MSAA
        // also we have to use the same shader for all objects during the Geometry Pass
        // to overcome some of these limitations we can use a hybrid approach
        // where we use deferred shading for opaque objects
        // and forward rendering for transparent objects and special effects

        // 1. geometry pass: render all geometric/color data to g-buffer 
        // NOTE: the G-buffer is a framebuffer which is not like a normal buffer (color , stencil or depth buffer)
        // the GPU has to store all these buffers somewhere in its memory this called a Framebuffer Object (FBO)
        // since it's a existing piece of data in GPU memory we can write to it and read from it
        // by default we already have a default framebuffer (with ID 0) created by the windowing system
        // but for deferred shading we need to create our own framebuffer which will contain our MRTs
        // creating and using the framebuffer is done in the same way as you would use a VAO or EBO
        // you bind buffer and then you can write to it or read from it
        // and at the end you unbind it
        unsafe {
            gl.bind_framebuffer(glow::FRAMEBUFFER, self.g_buffer);
            // black clear color and then we clear the color , depth and stencil buffers
            gl.clear_color(0.0, 0.0, 0.0, 1.0); 
            gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT | glow::STENCIL_BUFFER_BIT);
        }

        // Geometry Pass
        for object in &self.objects {
            object.draw(
                &self.camera.get_view_matrix(), 
                &self.camera.get_projection_matrix(),
                gl,
                resources,
                self.as_wireframe
            );
        }

        // Lighting Pass
        // Here we would use the information stored in the MRTs to calculate the final color of each pixel
        
        // because of the limitations of deferred shading
        // we would also need to do a forward rendering pass for transparent objects
        // so we have to copy the depth buffer from the G-buffer to the default framebuffer
        // to ensure proper depth testing during the forward rendering pass
        // unsafe {
        //     gl.bind_framebuffer(glow::FRAMEBUFFER, None);
        //     gl.clear_color(0.0, 0.0, 0.0, 1.0); 
        //     gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT | glow::STENCIL_BUFFER_BIT);
        // }

    
    }

    pub fn get_camera_mut(&mut self) -> &mut Camera {
        &mut self.camera
    }

    pub fn get_camera(&self) -> &Camera {
        &self.camera
    }


}

