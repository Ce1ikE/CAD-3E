use anyhow::Result;
use glow::{
    HasContext, 
    PixelUnpackData,
    Buffer,
    VertexArray,
};
use std::collections::HashMap;

// ============================================================================
// Vertex
// ============================================================================

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    pub m_position: [f32; 3],
    pub m_normal: [f32; 3],
    pub m_tex_coords: [f32; 2],
}

impl Vertex {
    pub fn new(m_position: [f32; 3], m_normal: [f32; 3], m_tex_coords: [f32; 2]) -> Self {
        Self {
            m_position ,
            m_normal,
            m_tex_coords,
        }
    }
}

// ============================================================================
// Mesh
// ============================================================================

pub struct Mesh {
    m_vao: VertexArray,
    m_vbo: Buffer,
    m_ebo: Option<Buffer>,
    m_index_count: usize,
    m_vertex_count: usize,
    m_is_indexed: bool,
}

impl Mesh {
    pub fn new(gl: &glow::Context, vertices: Vec<Vertex>, indices: Vec<u32>) -> Result<Self> {
        
        
        let is_indexed = indices.len() > 0;
        let mut index_count: usize = 0;
        let mut vertex_count: usize = 0;

        if is_indexed {
            // indexed drawing (using EBO)
            index_count = indices.len();
            vertex_count = 0;
        }
        else {
            // fallback to non-indexed drawing
            index_count = 0;
            vertex_count = vertices.len();
        }

        // sending data to the graphics card (a.k.a. GPU) from the CPU is relatively slow, 
        // so wherever we can, we try to send as much data as possible at once
        // We manage this memory via so called vertex buffer objects (VBO) 
        // that can store a large number of vertices in the GPU's memory. 
        // The advantage of using those buffer objects is that we can send large batches of data 
        // all at once to the graphics card, 
        // and keep it there if there's enough memory left (limited by video random access memory VRAM), 
        // without having to send data one vertex at a time.

    	// first we generate the buffers
        unsafe {

            let vao = gl
                .create_vertex_array()
                .map_err(|e| anyhow::anyhow!("Failed to create VAO: {}", e))?;
            
            let vbo = gl
                .create_buffer()
                .map_err(|e| anyhow::anyhow!("Failed to create VBO: {}", e))?;
            
            let mut ebo = None;
            if is_indexed {
                let ebo = gl
                    .create_buffer()
                    .map_err(|e| anyhow::anyhow!("Failed to create EBO: {}", e))?;
            }
            
            // next we bind the newly created buffers
	        // bind the Vertex Array Object first
            gl.bind_vertex_array(Some(vao));

            // then we copy our data to the buffers
            // The fourth parameter specifies how we want the graphics card to manage the given data. This can take 3 forms:
            // 1) GL_STREAM_DRAW	: the data is set only once and used by the GPU at most a few times
            // 2) GL_STATIC_DRAW	: the data is set only once and used many times
            // 3) GL_DYNAMIC_DRAW	: the data is changed a lot and used many times

            // upload vertices
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
            let vertex_data = std::slice::from_raw_parts(
                vertices.as_ptr() as *const u8,
                vertices.len() * std::mem::size_of::<Vertex>(),
            );
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, vertex_data, glow::STATIC_DRAW);

            if is_indexed {
                // upload indices only if provided 
                gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ebo.unwrap()));  

                let index_data = std::slice::from_raw_parts(
                    indices.as_ptr() as *const u8,
                    indices.len() * std::mem::size_of::<u32>(),
                );

                gl.buffer_data_u8_slice(glow::ELEMENT_ARRAY_BUFFER, index_data, glow::STATIC_DRAW);
            }
            // we sent the input vertex data to the GPU 
            // instructed the GPU how it should process the vertex data within a vertex and fragment shader however...
            // OpenGL does not yet know how it should interpret the vertex data in memory  (floats , ints, doubles , 2 or 3 or 4 values)
            // and how it should connect the vertex data to the vertex shader's attributes
            //
            // we can tell OpenGL how it should interpret the vertex data (per vertex attribute(positions , normals, etc...)) using
            // this following 2 lines basically say:
            // at location "0" of the buffer (VBO)
            // you'll find vertices with "3" "float" components
            // which should "not" be normalized (not clamped/cliped between -1.0 and 1.0)(they already are)
            // with a stride (byte offset) of "6" "of size" "float"
            // this data starts at location "0"

            // position attribute
            gl.vertex_attrib_pointer_f32(
                0,
                3,
                glow::FLOAT,
                false,
                std::mem::size_of::<Vertex>() as i32,
                0,
            );
            gl.enable_vertex_attrib_array(0);
            // same for the other attributes...
            // but the offset is different
            // normal attribute
            gl.vertex_attrib_pointer_f32(
                1,
                3,
                glow::FLOAT,
                false,
                std::mem::size_of::<Vertex>() as i32,
                std::mem::size_of::<[f32; 3]>() as i32,
            );
            gl.enable_vertex_attrib_array(1);
            // and again same for texture coordinates
            // Texture coordinate attribute
            gl.vertex_attrib_pointer_f32(
                2,
                2,
                glow::FLOAT,
                false,
                std::mem::size_of::<Vertex>() as i32,
                (std::mem::size_of::<[f32; 3]>() * 2) as i32,
            );
            gl.enable_vertex_attrib_array(2);
        	// more attributes : colors, tangents, bitangents, bone ids, weights, etc... 

            // VOB memory layout for 2 attributes (position,color) (all floats):
            // | ----- Vertex 1 ----- | ----- Vertex 2 ----- | ----- Vertex 2 ----- | ----- etc... ----- |
            //   X , Y , Z , R , G , B, X , Y , Z , R , G , B, X , Y , Z , R , G , B , etc...

            // you don't have to specify the "layout" of you're data you can query for the attribute locations in the OpenGL code 
            // however this way saves OpenGL calls and us as a programmer some work later on
            // more on VBO formatting and strategies : https://www.khronos.org/opengl/wiki/Vertex_Specification_Best_Practices

    	    // binding to 0 resets it to NULL
            gl.bind_buffer(glow::ARRAY_BUFFER, None);
            gl.bind_vertex_array(None);
            // do NOT unbind the EBO while a VAO is active
            // the EBO buffer object IS stored in the VAO
            // glBindBuffer(GL_ELEMENT_ARRAY_BUFFER, 0);

            // what is a VAO ?
            // what we did so far :
            // --------------------
            // (vertices -> VBO)
            // 1) copy our vertices array in a buffer for OpenGL to use 
            // 
            // (vertex shader + frag shader ->  shaderProgram)
            // 2) create a shader program by compiling shaders and linking them 
            // 
            // (dataShape)
            // 3) tell OpenGL how to interpret our current data inside the VBO
            // 
            // (draw object) 
            // 4) by using the shader program we can finally draw a object 

            // however there is a slight problem , we have to repeat this process every time we want to draw an object
            // the reason is because you're VBO is just an array of raw data that the GPU stores and can access for various processes
            // one of those "various processes" that the GPU can use them for is providing array data for vertex rendering operations
            // however there is no rule that says that a buffer you're currently using for vertex data cannot be used for other buffer usages too.
            // This is where a vertex array object (VAO) comes into play which is a object that stores 1 or more VBO's
            // and is designed to store the information (the state) necessary for drawing/rendering a object.
            // This reduces OpenGL calls of re-specifying the vertex format and bindings each time, you simply bind the VAO.

            Ok(Mesh {
                m_vao: vao,
                m_vbo: vbo,
                m_ebo: ebo,
                m_index_count: index_count,
                m_vertex_count: vertex_count,
                m_is_indexed: is_indexed,
            })
        }
    }

    pub fn draw(&self, gl: &glow::Context, draw_triangles: bool) {
        // once we specify what VAO we want to use to draw something then all the buffer pointers (that are pointing to VBO)
        // are called and passed trough the shaders that we have actived for this specific frame
        // remember that the transformation matrices are "uniform" variables defined inside the shader
        unsafe {
            gl.bind_vertex_array(Some(self.m_vao));

            if self.m_is_indexed {
                // indexed drawing
                if draw_triangles {
                    // draws the object as filled triangles
                    gl.draw_elements(glow::TRIANGLES, self.m_index_count as i32, glow::UNSIGNED_INT, 0);
                } else {
                    // draws the object as wireframe
                    gl.draw_elements(glow::LINES, self.m_index_count as i32, glow::UNSIGNED_INT, 0);
                }
            } else {
                // non-indexed drawing
                if draw_triangles {
                    gl.draw_arrays(glow::TRIANGLES, 0, self.m_vertex_count as i32);
                } 
                else {
                    gl.draw_arrays(glow::LINES, 0, self.m_vertex_count as i32);
                }
            }

            gl.bind_vertex_array(None);
        }
    }

    pub fn cleanup(&self, gl: &glow::Context) {
        unsafe {
            gl.delete_vertex_array(self.m_vao);
            gl.delete_buffer(self.m_vbo);

            if self.m_is_indexed {
                gl.delete_buffer(self.m_ebo.unwrap());
            }
        }
    }
}

// ============================================================================
// Shader
// ============================================================================

pub struct Shader {
    pub m_program: glow::Program,
}

impl Shader {
    pub fn new(gl: &glow::Context, vertex_src: &str, fragment_src: &str,geometry_src: Option<&str>) -> Result<Self> {
        unsafe {
            // it's quite simple to create a shader program since it's quite similar to how it's done in C++
            // the only difference is that instead of holding a shader program id (unsigned int)
            // we hold the actual program as a struct that glow provides us 
            // we'll apply the following steps:
            // (1) create the shader program
            // (2) foreach shader type :
            //      (2.1) create the shader
            //      (2.2) compile the shader
            //      (2.3) check for compilation errors
            //      (2.4) attach the shader to the program 
            // (3) delete the shaders as they're linked into the program and no longer necessary

            // (1)
            let program = gl.create_program()
            .map_err(|e| anyhow::anyhow!("Failed to create shader program: {}", e))?;

            let mut shader_types = vec![
                (glow::VERTEX_SHADER,vertex_src), 
                (glow::FRAGMENT_SHADER, fragment_src)
            ];

            if geometry_src.is_some() {
                shader_types.push((glow::GEOMETRY_SHADER, geometry_src.unwrap()));
            }

            let mut shaders: Vec<glow::Shader> = Vec::new();

            // (2)
            for (shader_type, shader_src) in shader_types {

                if shader_src.is_empty() {
                    continue;
                }
                // (2.1)
                let shader = gl.create_shader(shader_type)
                .map_err(|e| anyhow::anyhow!("Failed to create shader: {}", e))?;
                gl.shader_source(shader, shader_src);
                // (2.2)
                gl.compile_shader(shader);
                // (2.3)
                if !gl.get_shader_compile_status(shader) {
                    return Err(anyhow::anyhow!(
                        "Shader compilation failed: {}",
                        gl.get_shader_info_log(shader)
                    ));
                }
                // (2.4)
                gl.attach_shader(program, shader);
                shaders.push(shader);   
            }
            // normally in c++ we would keep the program id and use it later on
            // but here Glow makes it easier for us to manage a shader program as a struct
            gl.link_program(program);

            if !gl.get_program_link_status(program) {
                return Err(anyhow::anyhow!(
                    "Shader program linking failed: {}",
                    gl.get_program_info_log(program)
                ));
            }
            // finally we can delete the shaders as they're linked into the program now and no longer necessary
            for shader in shaders {
                gl.delete_shader(shader);
            }

            Ok(Self { m_program: program } )
        }
    }

    pub fn use_program(&self, gl: &glow::Context) {
        // used to activate the shader program during rendering
        // when we want to render an object with this shader program
        unsafe {
            gl.use_program(Some(self.m_program));
        }
    }

    // utility uniform functions we can use them to set uniform variables in the shader
    // during runtime, or query for their values if needed (like to display them in the editor UI for example)
    pub fn set_mat4(&self, gl: &glow::Context, name: &str, mat: &nalgebra_glm::Mat4) {
        unsafe {
            let location = gl.get_uniform_location(self.m_program, name);
            gl.uniform_matrix_4_f32_slice(location.as_ref(), false, mat.as_slice());
        }
    }

    pub fn set_vec2(&self, gl: &glow::Context, name: &str, vec: &nalgebra_glm::Vec2) {
        unsafe {
            let location = gl.get_uniform_location(self.m_program, name);
            gl.uniform_2_f32(location.as_ref(), vec.x, vec.y);
        }
    }

    pub fn set_vec3(&self, gl: &glow::Context, name: &str, vec: &nalgebra_glm::Vec3) {
        unsafe {
            let location = gl.get_uniform_location(self.m_program, name);
            gl.uniform_3_f32(location.as_ref(), vec.x, vec.y, vec.z);
        }
    }

    pub fn set_vec4(&self, gl: &glow::Context, name: &str, vec: &nalgebra_glm::Vec4) {
        unsafe {
            let location = gl.get_uniform_location(self.m_program, name);
            gl.uniform_4_f32(location.as_ref(), vec.x, vec.y, vec.z, vec.w);
        }
    }

    pub fn set_float(&self, gl: &glow::Context, name: &str, value: f32) {
        unsafe {
            let location = gl.get_uniform_location(self.m_program, name);
            gl.uniform_1_f32(location.as_ref(), value);
        }
    }

    pub fn set_int(&self, gl: &glow::Context, name: &str, value: i32) {
        unsafe {
            let location = gl.get_uniform_location(self.m_program, name);
            gl.uniform_1_i32(location.as_ref(), value);
        }
    }

    pub fn cleanup(&self, gl: &glow::Context) {
        unsafe {
            gl.delete_program(self.m_program);
        }
    }
}

// ============================================================================
// Texture
// ============================================================================

pub struct Texture2D {
    // holds the ID of the texture object, used for all texture operations to reference to this particular texture
    pub m_id: glow::Texture,
    // texture image dimensions
    // width and height of loaded image in pixels
    pub m_width: u32,
    pub m_height: u32,
    // format of texture object
    pub m_internal_format: u32,
    // format of loaded image
    pub m_image_format: u32,
    // texture configuration
    // wrapping mode on S axis
    m_wrap_s: u32,
    // wrapping mode on T axis
    m_wrap_t: u32, 
    // filtering mode if texture pixels < screen pixels
    m_filter_min: u32, 
    // filtering mode if texture pixels > screen pixels
    m_filter_max: u32, 
}

impl Texture2D {

    pub fn from_file(
        gl: &glow::Context, 
        path: &str,
        internal_format: u32,
        image_format: u32,
        wrap_s_mode: u32,
        wrap_t_mode: u32,
        filter_min_mode: u32,
        filter_max_mode: u32,
    ) -> Result<Self> {
        
        let img = image::open(path)
        .map_err(|e| anyhow::anyhow!("Failed to load texture {}: {}", path, e))?;
        
        let img = img.to_rgba8();
        let (width, height) = img.dimensions();

        Self::from_rgba_bytes(
            gl, 
            &img.into_raw(), 
            width, 
            height,
            internal_format,
            image_format,
            wrap_s_mode,
            wrap_t_mode,
            filter_min_mode,
            filter_max_mode,
        )
    }

    pub fn from_rgba_bytes(
        gl: &glow::Context,
        data: &[u8],
        width: u32,
        height: u32,
        internal_format: u32,
        image_format: u32,
        wrap_s_mode: u32,
        wrap_t_mode: u32,
        filter_min_mode: u32,
        filter_max_mode: u32,
    ) -> Result<Self> {
        unsafe {
            let texture = gl.create_texture()
            .map_err(|e| anyhow::anyhow!("Failed to create texture: {}", e))?;

            // create Texture by binding it first (same as VBO, VAO, EBO, Shader program, etc...)
            gl.bind_texture(glow::TEXTURE_2D, Some(texture));
            // set Texture wrap and filter modes (see Learn OpenGL for more info)
            gl.tex_parameter_i32(glow::TEXTURE_2D,glow::TEXTURE_WRAP_S,glow::REPEAT as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D,glow::TEXTURE_WRAP_T,glow::REPEAT as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D,glow::TEXTURE_MIN_FILTER,glow::LINEAR_MIPMAP_LINEAR as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D,glow::TEXTURE_MAG_FILTER,glow::LINEAR as i32);

            // tex_image_2d is used to specify a two-dimensional texture image,
            // it expects PixelUnpackData which is basically raw image data in bytes
            // to convert our raw data into that format we use Some(data)
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA as i32,
                width as i32,
                height as i32,
                0,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                PixelUnpackData::Slice(Some(&data)),
            );
            // a mimmap is a smaller, pre-filtered version of the original texture
            // used when the texture is viewed from a distance or at a small size
            // it improves performance and visual quality by reducing aliasing and moiré patterns
            // if you've seen the golden rectangle before then it kind of works like that 
            gl.generate_mipmap(glow::TEXTURE_2D);
            
            // unbind texture
            gl.bind_texture(glow::TEXTURE_2D, None);

            Ok(Self {
                m_id: texture,
                m_width: width,
                m_height: height,
                m_internal_format: internal_format,
                m_image_format: image_format,
                m_wrap_s: wrap_s_mode,
                m_wrap_t: wrap_t_mode,
                m_filter_min: filter_min_mode,
                m_filter_max: filter_max_mode,
            })
        }
    }

    pub fn bind(&self, gl: &glow::Context, unit: u32) {
        unsafe {
            gl.active_texture(glow::TEXTURE0 + unit);
            gl.bind_texture(glow::TEXTURE_2D, Some(self.m_id));
        }
    }

    pub fn cleanup(&self, gl: &glow::Context) {
        unsafe {
            gl.delete_texture(self.m_id);
        }
    }
}

// ============================================================================
// Material
// ============================================================================

// a Material is nothing but a collection of properties that define how an object surface interacts with light
// in practice this is just a bunch of float values and optionally a texture map
// that affect the final appearance of the object when rendered by inputting those values into the shader
// so basically they are input to the vertex and fragment shaders (aka uniforms)
// so we can use the Material of a object to set the shader uniforms before drawing the object
// object -> mesh -> vertex data -> vertex shader
// object -> material -> shader uniforms -> fragment shader

#[derive(Clone)]
pub struct Material {
    pub albedo: [f32; 3],
    pub metallic: f32,
    pub roughness: f32,
    pub ao: f32,
    pub albedo_texture: Option<String>,
    pub shader: Option<String>,
}

impl Material {
    pub fn new() -> Self {
        Self {
            albedo: [1.0, 1.0, 1.0],
            metallic: 0.0,
            roughness: 0.5,
            ao: 1.0,
            albedo_texture: None,
            shader: None,
        }
    }

    pub fn with_texture(mut self, texture_name: &str) -> Self {
        self.albedo_texture = Some(texture_name.to_string());
        self
    }

    pub fn with_shader(mut self, shader_name: &str) -> Self {
        self.shader = Some(shader_name.to_string());
        self
    }

    pub fn set_metallic(&mut self, metallic: f32) {
        self.metallic = metallic;
    }

    pub fn set_roughness(&mut self, roughness: f32) {
        self.roughness = roughness;
    }

    pub fn set_ao(&mut self, ao: f32) {
        self.ao = ao;
    }

    pub fn set_albedo(&mut self, albedo: [f32; 3]) {
        self.albedo = albedo;
    }
}

// ============================================================================
// Resources
// ============================================================================

pub struct Resources {
    // resource storage
    shaders: HashMap<String, Shader>,
    textures: HashMap<String, Texture2D>,
    meshes: HashMap<String, Mesh>,
    materials: HashMap<String, Material>,
}

impl Resources {
    pub fn new() -> Self {
        Self {
            shaders: HashMap::new(),
            textures: HashMap::new(),
            meshes: HashMap::new(),
            materials: HashMap::new(),
        }
    }

    // Shader management ========================
    // loads (and generates) a shader program from file loading vertex, fragment (and geometry) shader's source code. 
    // If geometry_src is not null, it also loads a geometry shader
    pub fn add_shader(
        &mut self,
        gl: &glow::Context,
        shader_name: &str,
        vertex_src: &str,
        fragment_src: &str,
        geometry_src: Option<&str>,
    ) -> Result<String> {
        let shader = Shader::new(gl, vertex_src, fragment_src, geometry_src)?;
        self.shaders.insert(shader_name.to_string(), shader);
        Ok(shader_name.to_string())
    }

    pub fn get_shader(&self, shader_name: &str) -> Option<&Shader> {
        self.shaders.get(shader_name)
    }
    
    // Texture management ========================
    // loads (and generates) a texture from file
    pub fn add_texture_from_file(
        &mut self, 
        gl: &glow::Context,
        texture_name: &str, 
        path: &str
    ) -> Result<String> {
        let texture = Texture2D::from_file(
            gl, 
            path,
            glow::RGBA,
            glow::RGBA,
            glow::REPEAT,
            glow::REPEAT,
            glow::LINEAR_MIPMAP_LINEAR,
            glow::LINEAR,
        )?;
        self.textures.insert(texture_name.to_string(), texture);
        Ok(texture_name.to_string())
    }

    pub fn add_texture_from_bytes(
        &mut self,
        gl: &glow::Context,
        texture_name: &str,
        data: &[u8],
        width: u32,
        height: u32,
    ) -> Result<String> {
        let texture = Texture2D::from_rgba_bytes(
            gl, 
            data, 
            width, 
            height,
            glow::RGBA,
            glow::RGBA,
            glow::REPEAT,
            glow::REPEAT,
            glow::LINEAR_MIPMAP_LINEAR,
            glow::LINEAR,
        )?;
        self.textures.insert(texture_name.to_string(), texture);
        Ok(texture_name.to_string())
    }

    pub fn get_texture(&self, texture_name: &str) -> Option<&Texture2D> {
        self.textures.get(texture_name)
    }

    // Mesh management ========================
    pub fn add_mesh(
        &mut self,
        gl: &glow::Context,
        mesh_name: &str,
        vertices: Vec<Vertex>,
        indices: Vec<u32>,
    ) -> Result<String> {
        let mesh = Mesh::new(gl, vertices, indices)?;
        self.meshes.insert(mesh_name.to_string(), mesh);
        Ok(mesh_name.to_string())
    }

    pub fn get_mesh(&self, mesh_name: &str) -> Option<&Mesh> {
        self.meshes.get(mesh_name)
    }

    // Material management ========================
    pub fn add_material(
        &mut self, 
        material_name: &str,
        material: Material
    ) -> Result<String> {
        self.materials.insert(material_name.to_string(), material);
        Ok(material_name.to_string())
    }

    pub fn get_material(&self, material_name: &str) -> Option<&Material> {
        self.materials.get(material_name)
    }

    // Cleanup ========================
    pub fn cleanup(&self, gl: &glow::Context) {
        for shader in self.shaders.values() {
            shader.cleanup(gl);
        }
        for texture in self.textures.values() {
            texture.cleanup(gl);
        }
        for mesh in self.meshes.values() {
            mesh.cleanup(gl);
        }
    }
}
