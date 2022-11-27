use std::mem::{self};

use glam::{Vec3, IVec3};

use crate::camera::{BoundingBox, AABB};

use super::{terrain::TerrainGenerator, geometry::{mesh::{Mesh}, voxel::{Voxel}, voxel_vertex::VoxelVertex, meshing::chunk_mesher::{ChunkMesher, VOXEL_SIZE}}};

pub const CHUNK_SIZE_X : usize = 20; // Should be equal to Z
pub const CHUNK_SIZE_Y : usize = 100;
pub const CHUNK_SIZE_Z : usize = 20;

pub const CHUNK_SIZE: [usize;3] = [CHUNK_SIZE_X,CHUNK_SIZE_Y,CHUNK_SIZE_X];

pub struct Chunk
{
    pub voxels: [[[Voxel; CHUNK_SIZE_Z] ; CHUNK_SIZE_Y] ; CHUNK_SIZE_X],
    pos: Vec3, // position in chunk space
    pub mesh: Option<Mesh<VoxelVertex>>,
}

impl Chunk
{
    /// Lazily create the Chunk, no mesh is created
    pub fn new(pos_x : i32 , pos_y : i32 , pos_z: i32 , generator: &dyn TerrainGenerator) -> Chunk
    {
        let mut voxels = [[[Voxel::default() ; CHUNK_SIZE_Z] ; CHUNK_SIZE_Y] ; CHUNK_SIZE_X];

        // chunk position offset in the world
        let x_offset = pos_x * CHUNK_SIZE_X as i32  ;
        let y_offset = pos_y * CHUNK_SIZE_Y as i32  ;
        let z_offset = pos_z * CHUNK_SIZE_Z as i32 ;

        // iterate over the voxels, requesting the type of each from the terrain generator
        for (x, x_row) in voxels.iter_mut().enumerate()
        {
            for (y, y_row) in x_row.iter_mut().enumerate()
            {
                for (z, voxel) in y_row.iter_mut().enumerate()
                {
                    generator.generate(voxel , x as i32 + x_offset , y as i32 + y_offset,z as i32 + z_offset);
                }
            }
        }

        Chunk{ pos: Vec3::new(pos_x as f32,pos_y as f32,pos_z as f32) , voxels , mesh: None::<Mesh<VoxelVertex>>}
    }

    /// Generate the chunk mesh
    pub fn generate_mesh<T> (&mut self)
        where T: ChunkMesher
    {
        self.mesh = Some(T::generate_mesh(self));
    }

    pub fn get_voxel(&self, pos: IVec3) -> Option<Voxel>
    {
        // make sure the pos is within bounds
        if pos.x < 0 || pos.x >= CHUNK_SIZE_X as i32  ||
             pos.y < 0 || pos.y >= CHUNK_SIZE_Y as i32  ||
             pos.z < 0 || pos.z >= CHUNK_SIZE_Z as i32
        {
            return None;
        }

        Some(self.voxels[pos.x as usize][pos.y as usize][pos.z as usize])
    }

    //TODO: duplicate code, refactor
    pub fn set_voxel(&mut self, pos: IVec3 ,voxel: Voxel)
    {
        // make sure the pos is within bounds
        if pos.x < 0 || pos.x >= CHUNK_SIZE_X as i32  ||
        pos.y < 0 || pos.y >= CHUNK_SIZE_Y as i32  ||
        pos.z < 0 || pos.z >= CHUNK_SIZE_Z as i32
        {
            return; // out of bounds, don't do anything
        }

        self.voxels[pos.x as usize][pos.y as usize][pos.z as usize] = voxel;
    }

    pub fn pos_chunk_space(&self) -> Vec3 { self.pos }

    pub fn pos_world_space(&self) -> Vec3 { Vec3::new((self.pos.x as i32 * CHUNK_SIZE_X as i32 ) as f32 ,
             (self.pos.y as i32 * CHUNK_SIZE_Y as i32 ) as f32 ,
                 (self.pos.z as i32 * CHUNK_SIZE_Z as i32 ) as f32 ) } 

    /// Returns the size in bytes on the chunk, the size of the mesh is excluded
    pub fn get_size_bytes(&self) -> usize
    {
        CHUNK_SIZE_X * CHUNK_SIZE_Y * CHUNK_SIZE_Z * mem::size_of::<Voxel>()
    }

}

impl BoundingBox for Chunk
{
    fn get_aabb(&self) -> AABB
    {
        // Calculate the AABB for the chunk
        let size = Vec3::new(CHUNK_SIZE_X as f32, CHUNK_SIZE_Y as f32, CHUNK_SIZE_Z as f32) * VOXEL_SIZE;
        let world_pos = self.pos * size;
        let ret = AABB::new(world_pos, world_pos + size);

        // println!("AAAB min {} max {}", ret.min, ret.max);

        ret
    }
}