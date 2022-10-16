use glam::{Vec3, IVec3};

use super::{terrain::TerrainGenerator, voxel::{Voxel, VOXEL_FACE_VALUES}, animation::ChunkMeshAnimation, geometry::{mesh::{Mesh}}};
use super::voxel::VoxelVertex;

pub const CHUNK_X : usize = 20;
pub const CHUNK_Z : usize = 20;
pub const CHUNK_Y : usize = 100;

pub struct Chunk
{
    pub voxels: [[[Voxel; CHUNK_Z] ; CHUNK_Y] ; CHUNK_X],
    pos: Vec3, // position in chunk space
    pub mesh: Option<Mesh<VoxelVertex>>,

    // animation
    pub animation: Option<ChunkMeshAnimation>
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

        Chunk{ pos: Vec3::new(pos_x as f32,pos_y as f32,pos_z as f32) , voxels , mesh: None::<Mesh<VoxelVertex>> , animation: None}
    }

    /// Generate the chunk mesh
    pub fn generate_mesh(&mut self)
    {
        // Generate the mesh
        let mut mesh: Mesh<VoxelVertex> = Mesh::new();
        
        //Generate the directly in here, good enough for now
        // for now render the mesh of all the voxels as is
        for x in 0..CHUNK_X
        {
            for y in 0..CHUNK_Y
            {
                for z in 0..CHUNK_X
                {
                    if self.voxels[x][y][z].is_filled() // not an air block
                    {
                        let mut faces_to_render: [bool;6] = [false;6];

                        // TODO: refactor
                        for (index, offset) in VOXEL_FACE_VALUES.iter().enumerate()
                        {
                            // is the neighbor of the current voxel in the given direction filled ? 
                            let pos = IVec3::new(x as i32 + offset.0,y as i32 + offset.1,z as i32 + offset.2);
                            if let Some(neighbor) = self.get_voxel(pos.x, pos.y, pos.z)
                            {
                                if !neighbor.is_filled()
                                {
                                    faces_to_render[index] = true ;
                                }
                            }
                            else
                            {
                                faces_to_render[index] = true;    
                            }
                        }

                        self.voxels[x][y][z].append_mesh_faces( &faces_to_render ,
                                self.pos_world_space() + Vec3::new(x as f32,y as f32,z as f32),
                                &mut mesh);
                    }
                }
            }
        }
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

    pub fn add_animation(&mut self , animation: ChunkMeshAnimation)
    {
        //fixme: what if we already have an animation ?
        self.animation = Some(animation);
    }

    // TODO: refactor
    pub fn update_animation(&mut self) -> bool 
    {
        let mut end = false;
        if let Some(animation) = self.animation.as_mut()
        {
            end = animation.update();
        }

        if end
        {
            self.animation = None; // remove animation
        }

        end 
    }

    pub fn pos_chunk_space(&self) -> Vec3 { self.pos }

    pub fn pos_world_space(&self) -> Vec3 { Vec3::new((self.pos.x as i32 * CHUNK_X as i32 ) as f32 ,
             (self.pos.y as i32 *CHUNK_Y as i32 ) as f32 ,
                 (self.pos.z as i32 * CHUNK_Z as i32 ) as f32 ) }

}