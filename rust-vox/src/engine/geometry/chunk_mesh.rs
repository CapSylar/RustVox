use glam::{Vec3, IVec2};
use super::{mesh::Mesh, voxel_vertex::VoxelVertex, meshing::{chunk_mesher::ChunkMesher, voxel_fetcher::VoxelFetcher}};

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

pub struct ChunkMesh
{
    pub mesh: Mesh<VoxelVertex>, // holds all geometry
    pub trans_faces: Vec<Face>, // holds references into the transparent faces stored in the mesh, used for transparency sorting
}

impl ChunkMesh
{
    /// Generates the chunk mesh
    pub fn new<T> (chunk_pos: IVec2, voxel_fetcher: VoxelFetcher) -> Self
    where T: ChunkMesher
    {
        let mut mesh = Mesh::<VoxelVertex>::default();
        let mut trans_faces = Vec::new();

        // mesh opaque geometry
        T::generate_mesh(chunk_pos, voxel_fetcher, &mut mesh, &mut trans_faces);
        // mesh transparent geometry

        Self{mesh, trans_faces}
    }

    /// Sort the transparent Faces with w.r.t their distances from pos
    pub fn sort_transparent(&mut self, center: Vec3)
    {
        // calculate the distance from pos for each face

        for face in self.trans_faces.iter_mut()
        {
            face.distance = face.pos.distance(center);
        }

        self.trans_faces.sort_by(|a, b| b.distance.total_cmp(&a.distance));

        // move the indices from nearest to furthest in the index buffer
        // indices of opaque geometry is sent to the back

        // new indices list
        let mut new_indices = Vec::new();
        let num_trans = self.get_num_trans_indices();

        new_indices.reserve(num_trans);
        
        for (face_index, face) in self.trans_faces.iter_mut().enumerate()
        {
            new_indices.extend_from_slice(&self.mesh.indices[face.base_index..face.base_index+6]); // get all 6 indices that form a face
            // the faces's indices have been moved, update their base index
            face.base_index = face_index * 6;
        }

        // copy the sorted indices list into the mesh
        self.mesh.indices[..num_trans].copy_from_slice(&new_indices[..]);

        // println!("after sorting");
        // for face in self.trans_faces.iter()
        // {
        //     print!("face is {:?} => indices: ", face);
        //     for index in self.mesh.indices[face.base_index..face.base_index+6].iter()
        //     {
        //         print!("{} ", index);
        //     }
        //     println!()
        // }

    }

    pub fn is_mesh_alloc(&self) -> bool
    {
        self.mesh.is_alloc()
    }

    pub fn get_num_trans_indices(&self) -> usize
    {
        self.trans_faces.len() * 6 // 6 indices per face (quad)
    }

    pub fn get_num_opaque_indices(&self) -> usize
    {
        self.mesh.indices.len() - self.get_num_trans_indices()
    }
}

