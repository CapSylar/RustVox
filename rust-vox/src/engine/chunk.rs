use glam::Vec3;

use super::{voxel::Voxel, mesh::{Mesh, self}};

const CHUNK_X : usize = 20;
const CHUNK_Z: usize = 20;
const CHUNK_Y : usize = 20;

pub struct Chunk
{
    //TODO: shouldn't this be on the heap? 
    voxels: [[[Voxel; CHUNK_Z] ; CHUNK_Y] ; CHUNK_X ],
    pub mesh: Mesh,
}

impl Chunk
{
    pub fn new() -> Chunk
    {
        let chunk = Chunk{ voxels: [[[Voxel::new_default() ; CHUNK_Z] ; CHUNK_Y] ; CHUNK_X ] , mesh:Mesh::new()};
        chunk
    }

    /// Generates the Mesh needed to render this chunk
    pub fn generate_mesh(&mut self) -> &Mesh
    {
        // for now render the mesh of all the voxels as is
        for x in 0..CHUNK_X
        {
            for y in 0..CHUNK_Y
            {
                for z in 0..CHUNK_Z
                {
                    self.voxels[x][y][z].append_mesh(Vec3::new(x as _ ,y as _,z as _ ), &mut self.mesh);
                }
            }
        }

        &self.mesh
    }

}