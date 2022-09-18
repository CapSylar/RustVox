use glam::Vec3;

pub struct ChunkMeshAnimation
{
    pub current: Vec3,
    // end position offset is always 0,0,0
    interpolation: f32, // 0 < interpolation < 1
    // chunk: Rc<RefCell<Chunk>>, // chunk to be animated
}

impl ChunkMeshAnimation
{
    pub fn new() -> Self // sensible defaults for now
    {
        Self
        {
            current: Vec3::new(0.0,-10.0,0.0),
            interpolation: 0.1,
        }
    }

    /// Get current position of chunk in animation
    pub fn get_pos(&self) -> Vec3
    {
        self.current
    }

    /// Update the current position
    pub fn update(&mut self) -> bool
    {
        let origin = Vec3::ZERO;
        // interpolate between current and end positions
        self.current = self.current + (origin - self.current) * self.interpolation;

        // debug 
        // println!("[DEBUG] new position of chunk animation: x:{},y:{},z:{}",
        //     self.current.x , self.current.y , self.current.z );
    
        (self.current - origin).length() <= 0.01 // close to endpoint, exit
    }
}