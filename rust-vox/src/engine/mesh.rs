use std::mem::size_of;

use glam::{Vec3, Vec2};


/// Contains everything we need to render geometry to the screen, namely the actual *vertices* and indices which
/// indicate how to construct triangles from the vertices
pub struct Mesh
{
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

/// Holds all the data which constitute a *vertex*
#[repr(C,packed)]
#[derive(Debug)]
pub struct Vertex
{
    position: Vec3, // X, Y , Z
    normal: Vec3, // X , Y , Z 
    tex_coord: Vec2, // U ,V
}

impl Vertex
{
    pub fn new( position: Vec3 , normal : Vec3 , texture_coordinate: Vec2 ) -> Vertex
    {
        Vertex { position, normal , tex_coord: texture_coordinate }
    }
}

impl Mesh
{
    pub fn new() -> Mesh
    {
        Mesh {vertices: Vec::<Vertex>::new(), indices: Vec::<u32>::new() }
    }

    pub fn add_vertex(&mut self, vertex: Vertex) -> u32
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

    pub fn add_trig_last_triple(&mut self)
    {
        // makes a triangle out of the last three added vertices
        todo!()       
    }

    pub fn size_bytes(&self) -> usize
    {
        self.vertices.len() * size_of::<Vertex>()
    }

    pub fn num_triangles(&self) -> usize
    {
        self.indices.len()
    }

}