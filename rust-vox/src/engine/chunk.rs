use glam::Vec3;

use super::{voxel::Voxel, mesh::{Mesh}};

pub const CHUNK_X : usize = 20;
pub const CHUNK_Z: usize = 20;
pub const CHUNK_Y : usize = 20;

pub struct Chunk
{
    //TODO: shouldn't this be on the heap? 
    voxels: [[[Voxel; CHUNK_Z] ; CHUNK_Y] ; CHUNK_X],
    pos: Vec3,
    pub mesh: Mesh,
}

impl Chunk
{
    pub fn new(pos: Vec3) -> Chunk
    {
        let voxels = [[[Voxel::new_default() ; CHUNK_Z] ; CHUNK_Y] ; CHUNK_X];
        let mesh = Mesh::new(&voxels, pos);

        let chunk = Chunk{ pos , voxels , mesh};
        chunk
    }
}