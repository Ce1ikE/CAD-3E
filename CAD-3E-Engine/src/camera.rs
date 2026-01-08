use nalgebra_glm as glm;

pub struct Camera {
    // https://learnopengl.com/img/getting-started/camera_axes.png
	// m_camera_position describes the position (point) of the camera in worldspace
    m_camera_position: glm::Vec3,
    // m_camera_target describes the position (point) towards what the camera is aimed at in worldspace 
    m_camera_target: glm::Vec3,
    // m_camera_front describes a unit vector from m_camera_position towards where we're looking at derived from m_yaw and m_pitch
    m_camera_front: glm::Vec3,
    // this gives us a direction from the camera's frame of reference and not worldspace
	// for camera rotation
	m_yaw: f32,
	m_pitch: f32,
	// a constant vector indicating "up" in the world
	// required for the dot products to get other vectors (right , up) based upon only position and front
    m_up_world_space: glm::Vec3,
    // a vector perpendicular to m_camera_front_direction and m_up_world_space
    m_right_direction: glm::Vec3,
	// the camera's local "up" direction which is the cross product between m_cameraRightDirection m_cameraDirection
    m_camera_up: glm::Vec3,
    m_camera_right: glm::Vec3,
	// m_view & m_projection used by Shader objects to calculate their vertices in the vertex shader
    m_view: glm::Mat4,
    m_projection: glm::Mat4,

	// init values for how much the camera is affected by directional changes (rotate , translate)
    m_pan_sensitivity: f32,
    m_orbit_sensitivity: f32,
    m_zoom_sensitivity: f32,
    m_distance: f32,

    m_fov: f32,
    m_near_plane: f32,
    m_far_plane: f32,
}

impl Camera {
    pub fn new() -> Self {
        
        let mut camera = Self {
            m_camera_position: glm::vec3(0.0, 0.0, 0.0),
            m_camera_target: glm::vec3(0.0, 0.0, 0.0),
            m_camera_front: glm::vec3(0.0, 0.0, -1.0),
            m_camera_up: glm::vec3(0.0, 1.0, 0.0),
            m_camera_right: glm::vec3(1.0, 0.0, 0.0),
            m_up_world_space: glm::vec3(0.0, 1.0, 0.0),
            m_right_direction: glm::vec3(1.0, 0.0, 0.0),
            m_yaw: -90.0,
            m_pitch: 0.0,
            m_pan_sensitivity: 0.05,
            m_orbit_sensitivity: 0.8,
            m_zoom_sensitivity: 1.5,
            m_distance: 10.0,
            m_view: glm::Mat4::identity(),
            m_projection: glm::Mat4::identity(),
            m_fov: 45.0,
            m_near_plane: 0.1,
            m_far_plane: 100.0,
        };
        
        camera.update_view();
        
        camera
    }

    pub fn update_position(&mut self) {
        // to prevent glimbal lock/flipping
        // https://computergraphics.stackexchange.com/questions/12273/gimbal-lock-confusion
        // https://fliponline.blogspot.com/2007/04/quick-trick-gimbal-lock-just-ignore-it.html
        self.m_pitch = glm::clamp_scalar(self.m_pitch, -89.0, 89.0);
        
        let yaw_rad = self.m_yaw.to_radians();
        let pitch_rad = self.m_pitch.to_radians();
        // we calculate the position relative to target using spherical coordinates
        // so are target is in the center of the sphere while our position lies upon the sphere
        let x = self.m_distance * pitch_rad.cos() * yaw_rad.cos();
        let y = self.m_distance * pitch_rad.sin();
        let z = self.m_distance * pitch_rad.cos() * yaw_rad.sin();
        // https://docs.blender.org/manual/en/2.93/editors/3dview/navigate/walk_fly.html#id1
        self.m_camera_position = self.m_camera_target + glm::vec3(x,y,z);
    }

    pub fn process_orbit(&mut self, delta_x: f32, delta_y: f32) {
        // move mouse left/right for yaw (around Y-axis)
        self.m_yaw += delta_x * self.m_orbit_sensitivity;
        // move mouse up/down for pitch (around X-axis)
        self.m_pitch += delta_y * self.m_orbit_sensitivity;

        self.update_position();
        self.update_view();
    }

    pub fn process_pan(&mut self, delta_x: f32, delta_y: f32) {
        // to pan, we need the camera's current right and up vectors.
	    // these are derived from the front direction relative to the world up
        self.m_camera_front = glm::normalize(&(self.m_camera_target - self.m_camera_position));
        self.m_camera_right = glm::normalize(&glm::cross(&self.m_camera_front, &self.m_up_world_space));
        self.m_camera_up = glm::normalize(&glm::cross(&self.m_camera_right, &self.m_camera_front)); 

        // the reason to multiply delta_x and delta_y by -1 is that moving the mouse
        // to the right should move the scene to the left (and vice versa)
        // like grabbing and dragging the scene (same in Blender)
        // so there is no math logic for this, it's just for intuitive control
        let mut translation = glm::vec3(0.0, 0.0, 0.0); 
        translation += -self.m_camera_right * (-1.0 * delta_x * self.m_pan_sensitivity);
        translation += self.m_camera_up * (-1.0 * delta_y * self.m_pan_sensitivity);

        self.update_position();
        self.update_view();
    }

    pub fn process_zoom(&mut self, delta_zoom: f32) {
        self.m_distance -= delta_zoom * self.m_zoom_sensitivity;
        
        if self.m_distance < 1.0 {
            self.m_distance = 1.0;
        }
        
        self.update_position();
        self.update_view();
    }

    pub fn update_projection(&mut self, width: f32, height: f32) {

        // TODO: switch between perspective 
        // update_projection() {
        // 
        // match self.m_projection_type {
        //    ProjectionType::Perspective => {
        //       m_projection = glm::perspective( ... );
        //    },
        //    ProjectionType::Orthographic => {
        //       m_projection = glm::ortho( ... );
        //    }
        // }

        // orthographic:
        // -------------
        // https://glm.g-truc.net/0.9.9/api/a00665.html#ga6615d8a9d39432e279c4575313ecb456
        // param 1 & 2 => left and right coordinate of the frustum 
        // param 3 & 4 => bottom and top coordinate of the frustum 
        // param 5 & 6 => distance between the near and far plane 
        // m_projection = glm::ortho(0.0f, (float)WINDOW_STD_HEIGHT , 0.0f, (float)WINDOW_STD_WIDTH, 0.1f, 100.0f);

        // perspective:
        // ------------
        // https://glm.g-truc.net/0.9.9/api/a00665.html#ga747c8cf99458663dd7ad1bb3a2f07787
        // param 1 => fov (field of view) value
        // param 2 => aspect ratio 
        // param 3 & 4 => distance between the near and far plane 
        self.m_projection = glm::perspective(
            self.m_fov.to_radians(), 
            width / height, 
            self.m_near_plane, 
            self.m_far_plane
        );
    }

    pub fn update_view(&mut self) {
        // takes the position from which you want to look at
        // target position you want to look at
        // and the vector which tells us what is up in worldspace
        self.m_view = glm::look_at(
            &self.m_camera_position,
            &self.m_camera_target,
            &self.m_up_world_space,
        );
    }

    pub fn update_camera(&mut self, delta_time: f32,keys: &bool) {
        // for interpolation between camera positions
        // https://www.opengl-tutorial.org/intermediate-tutorials/tutorial-17-quaternions/
        let camera_moved = true;

        if camera_moved {
            self.update_view();
        }
    }

    // getter methods
    pub fn get_view_matrix(&self) -> glm::Mat4 {
        self.m_view
    }

    pub fn get_projection_matrix(&self) -> glm::Mat4 {
        self.m_projection
    }

    pub fn get_position(&self) -> glm::Vec3 {
        self.m_camera_position
    }

    pub fn get_target(&self) -> glm::Vec3 {
        self.m_camera_target
    }

    // setter methods

    pub fn set_fov(&mut self, fov: f32, width: f32, height: f32) {
        self.m_fov = fov;
        self.update_projection(width, height);
    }

    pub fn set_near_plane(&mut self, near_plane: f32, width: f32, height: f32) {
        self.m_near_plane = near_plane;
        self.update_projection(width, height);
    }

    pub fn set_far_plane(&mut self, far_plane: f32, width: f32, height: f32) {
        self.m_far_plane = far_plane;
        self.update_projection(width, height);
    }

    pub fn set_position(&mut self, position: glm::Vec3) {
        self.m_camera_position = position;
        self.update_view();
    }

    pub fn set_target(&mut self, target: glm::Vec3) {
        self.m_camera_target = target;
        self.update_view();
    }

    pub fn set_distance(&mut self, distance: f32) {
        self.m_distance = distance;
        self.update_position();
        self.update_view();
    }

}
