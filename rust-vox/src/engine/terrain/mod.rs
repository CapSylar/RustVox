use noise::{Perlin, NoiseFn, Seedable};

use crate::engine::voxel::VoxelType;

use super::voxel::Voxel;

pub trait TerrainGenerator
{
    /// Determine the type of block that will reside at the specified x,y,z in the world \
    /// The x,y,z coordinates must be in world coordinates
    fn generate( &self, voxel: &mut Voxel , x:i32, y:i32, z:i32);
}

pub struct PerlinGenerator
{
    layer0: Perlin,
    layer1: Perlin,
}

impl PerlinGenerator
{
    pub fn new() -> Self
    {
        let layer0 = Perlin::new();
        let layer1 = Perlin::new();
        layer1.set_seed(2345345);
        Self{layer0, layer1}
    }
}

impl TerrainGenerator for PerlinGenerator
{
    fn generate( &self, voxel: &mut Voxel,  x:i32, y:i32, z:i32)
    {
        const MIN_HEIGHT: u32 = 10; // 10 blocks

        // println!("points we got, x:{} y:{} ", x as f64 * 10.0 , z as f64 * 10.0 );
        let weigth0 = self.layer0.get([x as f64 / 30.0, z as f64 / 30.0]);
        let weight1 = self.layer1.get([x as f64 / 10.0, z as f64 / 10.0]);

        let weight = 0.7;
        let max_height = (((weigth0 * weight + weight1 * (1.0 - weight)) + 1.0) * 15.0) as u32 + MIN_HEIGHT ;

        let y = y as u32 ;

        if y >= max_height
        {
            voxel.set_filled(false);
            return;
        }

        voxel.set_filled(true);

        if y >= 20
        {
            voxel.set_type(VoxelType::Grass);
        }
        else  // first 20 blocks are bedrock
        {
            voxel.set_type(VoxelType::Sand);
        }

    }
}