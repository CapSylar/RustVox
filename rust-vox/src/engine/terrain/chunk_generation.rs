use noise::{Perlin, NoiseFn};

use crate::engine::geometry::voxel::{Voxel, VoxelType};

// unsafe impl Sync for TerrainGenerator{}
pub trait TerrainGenerator : Sync
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

impl Default for PerlinGenerator
{
    fn default() -> Self
    {
        //TODO: use PlaneMapBuilder instead
        let layer0 = Perlin::new(2345345);
        let layer1 = Perlin::new(2345345);
        Self{layer0, layer1}
    }
}

impl TerrainGenerator for PerlinGenerator
{
    fn generate( &self, voxel: &mut Voxel,  x:i32, y:i32, z:i32)
    {
        const MIN_HEIGHT: u32 = 10; // 10 blocks

        let weigth0 = self.layer0.get([x as f64 / 30.0, z as f64 / 30.0]);
        let weight1 = self.layer1.get([x as f64 / 10.0, z as f64 / 10.0]);

        let weight = 0.7;
        let max_height = (((weigth0 * weight + weight1 * (1.0 - weight)) + 1.0) * 15.0) as u32 + MIN_HEIGHT ;

        let y = y as u32 ;

        if y >= max_height
        {
            if y <= 19
            {
                voxel.set_type(VoxelType::Water);
            }
            else
            {
                voxel.set_type(VoxelType::Air);
            }

            return;
        }

        if y >= 17
        {
            voxel.set_type(VoxelType::Dirt);
        }
        else  // first 20 blocks are bedrock
        {
            voxel.set_type(VoxelType::Sand);
        }

        // if y >= 10
        // {
        //     voxel.set_type(VoxelType::Air);
        // }
        // else {
        //     voxel.set_type(VoxelType::Dirt);
        // }

    }
}