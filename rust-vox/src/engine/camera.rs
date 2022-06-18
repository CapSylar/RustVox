use glam::{Vec3, Mat4};

pub struct Camera
{
    pub position: Vec3,
    pub front: Vec3,
    up: Vec3,

    speed: f32,

    yaw: f32,
    pitch: f32,
}

impl Camera
{
    /// Get a default camera at (0,0,3) looking towards -Z
    pub fn default() -> Camera
    {
        Camera
        {
            front: Vec3::new(0.0,0.0,-1.0),
            position: Vec3::new(0.0,0.0,3.0),
            up: Vec3::new(0.0,1.0,0.0),
            speed: 0.05,

            yaw: -89.9,
            pitch:0.0,
        }
    }

    pub fn new(position: Vec3 , front: Vec3 , up: Vec3 , speed: f32) -> Camera
    {
        Camera { position, front , up, speed , pitch:0.0,yaw:-89.9}
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

        //TODO: check these again
        self.front.x = f32::cos(self.yaw.to_radians()) * f32::cos(self.pitch.to_radians());
        self.front.y = f32::sin(self.pitch.to_radians());
        self.front.z = f32::sin(self.yaw.to_radians()) * f32::cos(self.pitch.to_radians());
    }

    pub fn get_look_at(&self) -> Mat4
    {
        Mat4::look_at_rh(self.position,self.position+self.front,self.up)
    }
}