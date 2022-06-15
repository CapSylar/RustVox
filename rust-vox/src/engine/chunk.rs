use glam::Vec3;

use super::{voxel::Voxel, mesh::{Mesh}, terrain::TerrainGenerator};

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
    pub fn new<T>(pos_x : u32 , pos_y : u32 , pos_z: u32 , generator: &T) -> Chunk
        where T: TerrainGenerator
    {
        let mut voxels = [[[Voxel::new_default() ; CHUNK_Z] ; CHUNK_Y] ; CHUNK_X];

        // chunk position offset in the world
        let x_offset = pos_x * CHUNK_X as u32 ;
        let y_offset = pos_y * CHUNK_Y as u32 ;
        let z_offset = pos_z * CHUNK_Z as u32 ;

        // iterate over the voxels, requesting the type of each from the generator
        for x in 0..CHUNK_X
        {
            for y in 0..CHUNK_Y
            {
                for z in 0..CHUNK_Z
                {
                    generator.generate(&mut voxels[x][y][z] , x as u32 + x_offset , y as u32 + y_offset,z as u32 + z_offset);
                }
            }
        }

        //FIXME: jesus christ
        let pos = Vec3::new((pos_x * CHUNK_X as u32 ) as f32 , (pos_y *CHUNK_Y as u32 ) as f32 , (pos_z * CHUNK_Z as u32 ) as f32 );
        // convert position from chunk space
        let mesh = Mesh::new(&voxels, pos);

        let chunk = Chunk{ pos , voxels , mesh};
        chunk
    }
}