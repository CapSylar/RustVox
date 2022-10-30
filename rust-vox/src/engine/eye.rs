use glam::{Vec3, Mat4};

pub struct Eye
{
    // projection parameters
    pub fov_y: f32,
    pub aspect_ratio: f32,
    pub near_plane: f32,
    pub far_plane: f32,
    // camera
    position: Vec3,
    front: Vec3,
    up: Vec3,

    speed: f32,

    // Euler Angles
    yaw: f32,
    pitch: f32,
}

impl Eye
{
    pub fn new(fov_y: f32, aspect_ratio: f32, near_plane: f32, far_plane: f32, position: Vec3 , front: Vec3 , up: Vec3 , speed: f32) -> Self
    {
        Self { fov_y, aspect_ratio, near_plane, far_plane, position, front , up, speed , pitch: 0.0, yaw: -89.9 }
    }

    pub fn get_position(&self) -> Vec3
    {
        self.position
    }

    pub fn get_front(&self) -> Vec3
    {
        self.front
    }

    pub fn move_forward(&mut self)
    {
        self.position += self.front * self.speed;
    }

    pub fn move_backward(&mut self)
    {
        self.position -= self.front * self.speed;
    }

    //FIXME: recalculating the cross every time ? 
    pub fn strafe_left(&mut self)
    {
        self.position -= Vec3::cross(self.front, self.up).normalize() * self.speed;
    }

    //FIXME: recalculating the cross every time ? 
    pub fn strafe_right(&mut self)
    {
        self.position += Vec3::cross(self.front, self.up).normalize() * self.speed;
    }

    /// Change the Camera's direction 
    pub fn change_front_rel( &mut self,x_rel: f32 , y_rel: f32 )
    {
        self.yaw += x_rel;
        self.pitch += -y_rel;

        // put a limit on pitch
        self.pitch = self.pitch.clamp(-89.9, 89.9);

        // recalculate front vector
        self.front.x = f32::cos(self.yaw.to_radians()) * f32::cos(self.pitch.to_radians());
        self.front.y = f32::sin(self.pitch.to_radians());
        self.front.z = f32::sin(self.yaw.to_radians()) * f32::cos(self.pitch.to_radians());
    }

    pub fn get_look_at(&self) -> Mat4
    {
        Mat4::look_at_rh(self.position,self.position+self.front,self.up)
    }

    pub fn get_persp_trans(&self) -> Mat4
    {
        Mat4::perspective_rh_gl(self.fov_y, self.aspect_ratio, self.near_plane, self.far_plane)
    }
}