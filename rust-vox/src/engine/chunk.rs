use std::{mem::{self}};

use glam::{Vec3, IVec3, IVec2};

use crate::camera::{BoundingBox, AABB};

use super::{terrain::TerrainGenerator, geometry::{mesh::{Mesh}, voxel::{Voxel}, voxel_vertex::VoxelVertex, meshing::chunk_mesher::{ChunkMesher, VOXEL_SIZE}}};

pub const CHUNK_SIZE_X : usize = 20; // Should be equal to Z
pub const CHUNK_SIZE_Y : usize = 100;
pub const CHUNK_SIZE_Z : usize = 20;

pub const CHUNK_SIZE: [usize;3] = [CHUNK_SIZE_X,CHUNK_SIZE_Y,CHUNK_SIZE_X];

#[derive(Debug)]
pub struct Face
{
    pos: Vec3, // position in world space
    base_index: usize, // index in to the indices list in meshes
    distance: f32 // used for sorting
}

impl Face
{
    pub fn new(pos: Vec3, indices: usize) -> Self
    {
        Self{pos,base_index: indices,distance:0.0}
    }
}

pub struct Chunk
{
    pub voxels: [[[Voxel; CHUNK_SIZE_Z] ; CHUNK_SIZE_Y] ; CHUNK_SIZE_X],
    pos: Vec3, // position in chunk space

    pub mesh: Option<Mesh<VoxelVertex>>, // holds all geometry
    pub trans_faces: Option<Vec<Face>>, // holds references into the transparent faces stored in the mesh, used for transparency sorting
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

        Chunk{ pos: Vec3::new(pos_x as f32,pos_y as f32,pos_z as f32) , voxels , mesh: None, trans_faces: None}
    }

    /// Generate the chunk mesh
    pub fn generate_mesh<T> (&mut self)
        where T: ChunkMesher
    {
        let mut mesh = Mesh::<VoxelVertex>::default();
        let mut faces = Vec::new();

        // mesh opaque geometry
        T::generate_mesh(self, &mut mesh, &mut faces);
        // mesh transparent geometry
        
        self.mesh = Some(mesh);
        self.trans_faces = Some(faces);
    }

    /// Sort the transparent Faces with w.r.t their distances from pos
    pub fn sort_transparent(&mut self, center: Vec3)
    {
        let mesh = self.mesh.as_mut().unwrap();
        // calculate the distance from pos for each face
        if self.trans_faces.is_none() // should not happen
        {
            panic!("Called sort_transparent() on a mesh that doesn't have transparent faces");
        }

        let faces = self.trans_faces.as_mut().unwrap();

        // for face in faces
        // {
        //     println!("face is {:?}", face);
        // }

        // println!("Indices count: {}", mesh.get_indices_len());

        for face in faces.iter_mut()
        {
            face.distance = face.pos.distance(center);
        }

        faces.sort_by(|a, b| a.distance.total_cmp(&b.distance));

        // move the indices from nearest to furthest in the index buffer
        // indices of opaque geometry is sent to the back

        // new indices list
        let mut new_indices = Vec::new();
        new_indices.reserve(faces.len() * 6);
        
        for face in faces.iter_mut()
        {
            new_indices.extend_from_slice(&mesh.indices[face.base_index..face.base_index+6]); // get all 6 indices that form a face
        }

        // copy the sorted indices list into the mesh
        mesh.indices[..faces.len() * 6].copy_from_slice(&new_indices[..]);

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

    pub fn get_mesh(&self) -> Option<&Mesh<VoxelVertex>>
    {
        self.mesh.as_ref()
    }

    pub fn is_mesh_alloc(&self) -> bool
    {
        match self.mesh.as_ref()
        {
            Some(mesh) => mesh.is_alloc(),
            None => false,
        }
    }

    // FIXME: refactor
    pub fn pos_chunk_space(&self) -> IVec2
    {
        IVec2::new( self.pos.x as i32, self.pos.z as i32)
    }

    pub fn pos_world_space(&self) -> Vec3 { Vec3::new((self.pos.x as i32 * CHUNK_SIZE_X as i32 ) as f32 ,
             (self.pos.y as i32 * CHUNK_SIZE_Y as i32 ) as f32 ,
                 (self.pos.z as i32 * CHUNK_SIZE_Z as i32 ) as f32 ) } 

    /// Returns the size in bytes on the chunk, the size of the mesh is excluded
    pub fn get_size_bytes(&self) -> usize
    {
        CHUNK_SIZE_X * CHUNK_SIZE_Y * CHUNK_SIZE_Z * mem::size_of::<Voxel>()
    }

    pub fn get_num_trans_indices(&self) -> usize
    {
        match &self.trans_faces
        {
            Some(faces) => faces.len() * 6,
            None => 0
        }
    }
    
    pub fn get_num_opaque_indices(&self) -> usize
    {
        match &self.mesh
        {
            Some(mesh) => mesh.indices.len() - self.get_num_trans_indices(),
            None => 0
        }
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