use glam::Vec3;

use super::{voxel::Voxel, mesh::{Mesh}};

pub const CHUNK_X : usize = 20;
pub const CHUNK_Z: usize = 20;
pub const CHUNK_Y : usize = 100;

pub struct Chunk
{
    //TODO: shouldn't this be on the heap? 
    voxels: [[[Voxel; CHUNK_Z] ; CHUNK_Y] ; CHUNK_X],
    pos: Vec3, // position in chunk space
    pub mesh: Mesh,
}

impl Chunk
{
    pub fn new(pos_x : u32 , pos_y : u32 , pos_z: u32) -> Chunk
    {
        let voxels = [[[Voxel::new_default() ; CHUNK_Z] ; CHUNK_Y] ; CHUNK_X];

        //FIXME: jesus christ
        let pos = Vec3::new((pos_x * CHUNK_X as u32 ) as f32 , (pos_y *CHUNK_Y as u32 ) as f32 , (pos_z * CHUNK_Z as u32 ) as f32 );
        // convert position from chunk space
        let mesh = Mesh::new(&voxels, pos);

        let chunk = Chunk{ pos , voxels , mesh};
        chunk
    }
}