use std::{cell::RefCell, rc::Rc};

use glam::{Vec3, Mat4};

use crate::DebugData;

pub struct Camera
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

    // Movement
    speed: f32,

    // Euler Angles
    yaw: f32,
    pitch: f32,

    // Frustum
    frustum: Frustum,

    // debug data
    debug_data: Rc<RefCell<DebugData>>
}

impl Camera
{
    pub fn new(fov_y: f32, aspect_ratio: f32, near_plane: f32, far_plane: f32, position: Vec3 , front: Vec3 , up: Vec3 , speed: f32, debug_data: &Rc<RefCell<DebugData>>) -> Self
    {
        let frustum =  Frustum::new(position, front, up, near_plane, far_plane, fov_y, aspect_ratio);

        let debug_data = debug_data.clone();
        debug_data.borrow_mut().player_pos = position;
        debug_data.borrow_mut().front = front;

        Self { fov_y, aspect_ratio, near_plane, far_plane, position, front , up, speed , pitch: 0.0, yaw: -89.9, frustum, debug_data}
    }

    pub fn get_position(&self) -> Vec3
    {
        self.position
    }

    pub fn set_position(&mut self, pos: Vec3 )
    {
        self.position = pos;
        self.rebuild_frustum();
        // update the debug data
        self.debug_data.borrow_mut().player_pos = self.position;
    }

    pub fn get_front(&self) -> Vec3
    {
        self.front
    }

    pub fn set_front(&mut self, front: Vec3)
    {
        self.front = front;
        self.rebuild_frustum();
        // update the debug data
        self.debug_data.borrow_mut().front = self.front;
    }

    pub fn move_forward(&mut self)
    {
        self.set_position(self.position + self.front * self.speed);
    }

    pub fn move_backward(&mut self)
    {
        self.set_position(self.position - self.front * self.speed);
    }

    pub fn strafe_left(&mut self)
    {
        self.set_position(self.position - Vec3::cross(self.front, self.up).normalize() * self.speed);
    }

    pub fn strafe_right(&mut self)
    {
        self.set_position(self.position + Vec3::cross(self.front, self.up).normalize() * self.speed);
    }

    /// Change the Camera's direction 
    pub fn change_front_rel(&mut self, x_rel: f32, y_rel: f32)
    {
        self.yaw += x_rel;
        self.pitch += -y_rel;

        // put a limit on pitch
        self.pitch = self.pitch.clamp(-89.9, 89.9);

        // recalculate front vector
        let front = Vec3::new(
            f32::cos(self.yaw.to_radians()) * f32::cos(self.pitch.to_radians()),
            f32::sin(self.pitch.to_radians()),
            f32::sin(self.yaw.to_radians()) * f32::cos(self.pitch.to_radians())
        );

        self.set_front(front);
    }

    pub fn get_look_at(&self) -> Mat4
    {
        Mat4::look_at_rh(self.position,self.position+self.front,self.up)
    }

    pub fn get_persp_trans(&self) -> Mat4
    {
        Mat4::perspective_rh_gl(self.fov_y, self.aspect_ratio, self.near_plane, self.far_plane)
    }

    pub fn rebuild_frustum(&mut self)
    {
        self.frustum = Frustum::new(self.position, self.front, self.up, self.near_plane, self.far_plane, self.fov_y, self.aspect_ratio);
    }

    pub fn is_visible<T> (&self, geometry: &T) -> bool
        where T: BoundingBox
    {
        self.frustum.intersect(&geometry.get_aabb())
    }
}

pub struct AABB
{
    pub min: Vec3,
    pub max: Vec3,
}

impl AABB
{
    pub fn new(min: Vec3, max: Vec3) -> Self
    {
        Self{min,max}
    }
}

pub trait BoundingBox
{
    fn get_aabb(&self) -> AABB;
}

pub struct Frustum
{
    // 6 frustum planes: near,far,left,right,top,bottom
    planes: [(Vec3,Vec3); 6],
}

impl Frustum
{
    pub fn new(position: Vec3, front: Vec3, up: Vec3, near: f32, far: f32, fov_y: f32, aspect_ratio: f32) -> Self
    {
        let right = Vec3::cross(front, up);

        let far_plane_h = fov_y.tan() * far;
        let far_plane_w = far_plane_h * aspect_ratio;

        let far_vec = front * far;

        let planes: [(Vec3,Vec3); 6] = [
            (position + near * front, front), // near plane
            (position + far_vec, -front), // far plane
            (position, Vec3::cross(up, far_vec + right * far_plane_w/2.0)), // right plane
            (position, Vec3::cross(far_vec - right * far_plane_w/2.0 ,up)), // left plane
            (position, Vec3::cross(far_vec + up * far_plane_h/2.0, right)), // top plane
            (position, Vec3::cross(right, far_vec - up * far_plane_h/2.0)), // bottom plane
        ];

        // dbg!(planes);
        
        Self{planes}
    }

    pub fn default() -> Self
    {
        Self {planes: [(Vec3::ZERO,Vec3::ZERO);6]}
    }

    /// Returns true if the AABB is partially/fully inside the frustum
    pub fn intersect(&self, bounding_box: &AABB) -> bool
    {
        let max = bounding_box.max;
        let min = bounding_box.min;

        self.planes.into_iter().all(|(pos, normal)| 
        {
            // get the bounding box corner that is the farthest in the direction of the plane normal
            let mut corner: Vec3 = Vec3::ZERO;

            corner.x = if normal.x > 0.0 {max.x} else {min.x};
            corner.y = if normal.y > 0.0 {max.y} else {min.y};
            corner.z = if normal.z > 0.0 {max.z} else {min.z};

            let corner_vec = corner - pos; // point on plane -> corner vector

            Vec3::dot(corner_vec, normal) > 0.0 // if positive, the corner is in front of the plane, meaning this test has passed
            // else it is completely behind the plane, stop further tests, it is outside the frustum
        })
    }
}