use rand::Rng;

use super::voxel::Voxel;

pub trait TerrainGenerator
{
    /// Determine the type of block that will reside at the specified x,y,z in the world \
    /// The x,y,z coordinates must be in world coordinates
    fn generate( &self, voxel: &mut Voxel , x:u32, y:u32, z:u32);
}

pub struct PerlinGenerator
{

}

impl PerlinGenerator
{
    pub fn new() -> Self
    {
        Self{}
    }
}

impl TerrainGenerator for PerlinGenerator
{
    fn generate( &self, voxel: &mut Voxel,  x:u32, y:u32, z:u32)
    {
        let mut rng = rand::thread_rng();
        voxel.set_filled(rng.gen());
    }
}