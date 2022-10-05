use std::mem::size_of;

use glam::{Vec3, Vec2, IVec3};
use super::{chunk::{Chunk, CHUNK_X, CHUNK_Y, CHUNK_Z}, voxel::VOXEL_FACE_VALUES, renderer::opengl_abstractions::{vertex_array::{VertexArray, VertexBufferLayout}, vertex_buffer::VertexBuffer, index_buffer::IndexBuffer}};

/// Contains everything we need to render geometry to the screen, namely the actual *vertices* and indices which
/// indicate how to construct triangles from the vertices
pub struct Mesh<T>
{
    pub vertices: Vec<T>,
    pub indices: Vec<u32>,
    pub vao: Option<VertexArray<T>>,
}

/// Holds all the data which constitute a *vertex*
#[repr(C,packed)]
#[derive(Clone,Copy,Debug)]
pub struct Vertex
{
    position: Vec3, // X, Y , Z
    tex_coord: Vec2, // U ,V
    normal_index : u8, // byte index into a normal LUT in the shader, 6 possible normal vectors
}

impl Vertex
{
    pub fn new( position: Vec3 , normal_index : u8 , texture_coordinate: Vec2 ) -> Vertex
    {
        Vertex { position , tex_coord: texture_coordinate, normal_index  }
    }
}

impl<T> Mesh<T>
{
    pub fn new() -> Self
    {
        Self {vertices:Vec::new(),indices:Vec::new(),vao:None}
    }

    pub fn from_chunk(chunk: &mut Chunk) -> Mesh<Vertex>
    {
        // Generate the mesh
        let mut mesh = Mesh{vertices:Vec::<Vertex>::new(),indices:Vec::<u32>::new(),vao: None};

        //Generate the directly in here, good enough for now
        // for now render the mesh of all the voxels as is
        for x in 0..CHUNK_X
        {
            for y in 0..CHUNK_Y
            {
                for z in 0..CHUNK_Z
                {
                    if chunk.voxels[x][y][z].is_filled() // not an air block
                    {
                        let mut faces_to_render: [bool;6] = [false;6];

                        // TODO: refactor
                        for (index, offset) in VOXEL_FACE_VALUES.iter().enumerate()
                        {
                            // is the neighbor of the current voxel in the given direction filled ? 
                            let pos = IVec3::new(x as i32 + offset.0,y as i32 + offset.1,z as i32 + offset.2);
                            if let Some(neighbor) = chunk.get_voxel(pos.x, pos.y, pos.z)
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

                        chunk.voxels[x][y][z].append_mesh_faces( &faces_to_render ,
                                chunk.pos_world_space() + Vec3::new(x as f32,y as f32,z as f32),
                                &mut mesh);
                    }
                }
            }
        }
        mesh
    }

    /// `Upload Mesh to the GPU`
    /// Creates the VAO,VBO and EBO needed render this chunk
    pub fn upload(&mut self)
    {
        // create the vertex buffer
        let vertex_buffer = VertexBuffer::new(&self.vertices);
         // create a vertex buffer layout
         let mut layout = VertexBufferLayout::new();
        
         layout.push_f32(3); // vertex(x,y,z)
         layout.push_f32(2); // uv coordinates(u,v)
         layout.push_u8(1); // Normal Index
         // create the index buffer
         let index_buffer = IndexBuffer::new(&self.indices);

         let vao = VertexArray::new(vertex_buffer,&layout, index_buffer);
         self.vao = Some(vao);
    }

    pub fn add_vertex(&mut self, vertex: T) -> u32
    {
        self.vertices.push(vertex);
        self.vertices.len() as u32 -1
    }

    pub fn add_triangle(&mut self, p1: u32 , p2:u32 , p3:u32 )
    {
        self.indices.push(p1);
        self.indices.push(p2);
        self.indices.push(p3);
    }

    pub fn size_bytes(&self) -> usize
    {
        self.vertices.len() * size_of::<Vertex>()
    }

}