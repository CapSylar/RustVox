use std::ops::Add;

#[derive(Copy,Clone)]
pub struct Vec3i32
{
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl Vec3i32
{
    pub const fn new(x:i32 , y:i32 , z:i32) -> Self
    {
        Self{x,y,z}
    }
}

impl Add for Vec3i32
{
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output
    {
        Self
        {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }

    
}
