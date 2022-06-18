use noise::{Perlin, NoiseFn};

use super::voxel::Voxel;

pub trait TerrainGenerator
{
    /// Determine the type of block that will reside at the specified x,y,z in the world \
    /// The x,y,z coordinates must be in world coordinates
    fn generate( &self, voxel: &mut Voxel , x:i32, y:i32, z:i32);
}

pub struct PerlinGenerator
{
    perlin: Perlin,
}

impl PerlinGenerator
{
    pub fn new() -> Self
    {
        let perlin = Perlin::new();
        Self{perlin}
    }
}

impl TerrainGenerator for PerlinGenerator
{
    fn generate( &self, voxel: &mut Voxel,  x:i32, y:i32, z:i32)
    {
        // println!("points we got, x:{} y:{} ", x as f64 * 10.0 , z as f64 * 10.0 );
        let max_height = self.perlin.get([x as f64 / 10.0, z as f64 / 10.0]) * 10.0 + 10.0 ;
        let max_height = max_height as i32;
        // println!("max height {}" , max_height);

        if y >= max_height
        {
            voxel.set_filled(false);
            return;
        }

        // first 20 blocks are bedrock
        voxel.set_filled(true);
    }
}