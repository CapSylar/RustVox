use glam::Vec3;

use super::{mesh::{Mesh}, terrain::TerrainGenerator, voxel::Voxel};

pub const CHUNK_X : usize = 20;
pub const CHUNK_Z: usize = 20;
pub const CHUNK_Y : usize = 100;

// pub const CHUNK_X_U32 : u32 = CHUNK_X as u32;
// pub const CHUNK_Y_U32 : u32 = CHUNK_Y as u32;
// pub const CHUNK_Z_U32 : u32 = CHUNK_Z as u32;

pub struct Chunk
{
    //TODO: shouldn't this be on the heap? 
    pub voxels: [[[Voxel; CHUNK_Z] ; CHUNK_Y] ; CHUNK_X],
    pub pos: Vec3, // position in chunk space
    pub mesh: Option<Mesh>,
}

impl Chunk
{
    /// Lazily create the Chunk, no mesh is created
    pub fn new(pos_x : i32 , pos_y : i32 , pos_z: i32 , generator: &dyn TerrainGenerator) -> Chunk
    {
        let mut voxels = [[[Voxel::new_default() ; CHUNK_Z] ; CHUNK_Y] ; CHUNK_X];

        // chunk position offset in the world
        let x_offset = pos_x * CHUNK_X as i32  ;
        let y_offset = pos_y * CHUNK_Y as i32  ;
        let z_offset = pos_z * CHUNK_Z as i32 ;

        // iterate over the voxels, requesting the type of each from the terrain generator
        for x in 0..CHUNK_X
        {
            for y in 0..CHUNK_Y
            {
                for z in 0..CHUNK_Z
                {
                    generator.generate(&mut voxels[x][y][z] , x as i32 + x_offset , y as i32 + y_offset,z as i32 + z_offset);
                }
            }
        }

        //FIXME: jesus christ
        let pos = Vec3::new((pos_x * CHUNK_X as i32 ) as f32 , (pos_y *CHUNK_Y as i32 ) as f32 , (pos_z * CHUNK_Z as i32 ) as f32 );
        // convert position from chunk space
        let chunk = Chunk{ pos , voxels , mesh: None};
        chunk
    }

    /// Generate the chunk mesh    
    pub fn create_mesh(&mut self)
    {
        let mesh = Mesh::new(self);
        // assign the created mesh
        self.mesh = Some(mesh);
    }

    pub fn get_voxel(&self, pos_x: i32 , pos_y:i32 , pos_z:i32) -> Option<Voxel>
    {
        // make sure the pos is within bounds
        if pos_x < 0 || pos_x >= CHUNK_X as i32  ||
             pos_y < 0 || pos_y >= CHUNK_Y as i32  ||
             pos_z < 0 || pos_z >= CHUNK_Z as i32
        {
            return None;
        }

        Some(self.voxels[pos_x as usize][pos_y as usize][pos_z as usize])
    }

}