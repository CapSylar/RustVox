use glam::{Vec3, Vec2};

use super::mesh::{Vertex, Mesh};

const VOXEL_SIZE: f32 = 1.0;

#[derive(Clone, Copy)]
pub enum VoxelType
{
    Grass,
}

#[derive(Copy,Clone)]
pub struct Voxel
{
    voxel_type : VoxelType,
    is_filled: bool,
}

impl Voxel
{
    pub fn new( voxel_type: VoxelType, is_filled: bool) -> Voxel
    {
        Voxel{voxel_type, is_filled}
    }

    pub fn new_default() -> Voxel
    {
        Voxel { voxel_type: VoxelType::Grass, is_filled: true }
    }

    pub fn set_filled(&mut self, filled: bool)
    {
        self.is_filled = filled;
    }

    /// Returns the Voxel's mesh for rendering
    pub fn append_mesh( &self , pos: Vec3 , mesh: &mut Mesh)
    {
        // generate the 8 vertices to draw the voxel
        //bottom
        let p1 = Vec3::new(pos.x + 0.0,pos.y + 0.0,pos.z + 0.0);
        let p2 = Vec3::new(pos.x + 0.0,pos.y + 0.0,pos.z + -VOXEL_SIZE);
        let p3 = Vec3::new(pos.x + VOXEL_SIZE,pos.y + 0.0,pos.z + -VOXEL_SIZE);
        let p4 = Vec3::new(pos.x + VOXEL_SIZE,pos.y + 0.0,pos.z + 0.0);

        //top
        let p5 = Vec3::new(pos.x + 0.0,pos.y + VOXEL_SIZE,pos.z + 0.0);
        let p6 = Vec3::new(pos.x + 0.0,pos.y + VOXEL_SIZE,pos.z + -VOXEL_SIZE);
        let p7 = Vec3::new(pos.x + VOXEL_SIZE,pos.y + VOXEL_SIZE,pos.z + -VOXEL_SIZE);
        let p8 = Vec3::new(pos.x + VOXEL_SIZE,pos.y + VOXEL_SIZE,pos.z + 0.0);

        let uv = Vec2::new(0.0,0.0);
        // add the 2 top triangles
        let normal = Vec3::new(0.0,1.0,0.0);
        let i1 = mesh.add_vertex(Vertex::new( p5, normal, uv));
        let i2 = mesh.add_vertex(Vertex::new( p6, normal, uv));
        let i3 = mesh.add_vertex(Vertex::new( p7, normal, uv));
        let i4 = mesh.add_vertex(Vertex::new( p8, normal, uv));
        // construct the 2 triangles
        mesh.add_triangle(i1,i2,i3);
        mesh.add_triangle(i1, i3, i4);

        // add the 2 bottom triangles
        let normal = Vec3::new(0.0,-1.0,0.0);
        let i1 = mesh.add_vertex(Vertex::new( p1, normal, uv));
        let i2 = mesh.add_vertex(Vertex::new( p2, normal, uv));
        let i3 = mesh.add_vertex(Vertex::new( p3, normal, uv));
        let i4 = mesh.add_vertex(Vertex::new( p4, normal, uv));
        // construct the 2 triangles, note the order of the vertices in the trigs
        mesh.add_triangle(i3,i2,i1);
        mesh.add_triangle(i4, i3, i1);

        // add the 2 front triangles
        let normal = Vec3::new(0.0,0.0,1.0);
        let i1 = mesh.add_vertex(Vertex::new( p1, normal, uv));
        let i2 = mesh.add_vertex(Vertex::new( p5, normal, uv));
        let i3 = mesh.add_vertex(Vertex::new( p8, normal, uv));
        let i4 = mesh.add_vertex(Vertex::new( p4, normal, uv));
        // construct the 2 triangles, note the order of the vertices in the trigs
        mesh.add_triangle(i1,i2,i3);
        mesh.add_triangle(i1, i3, i4);

        // add the 2 back triangles
        let normal = Vec3::new(0.0,0.0,-1.0);
        let i1 = mesh.add_vertex(Vertex::new( p2, normal, uv));
        let i2 = mesh.add_vertex(Vertex::new( p6, normal, uv));
        let i3 = mesh.add_vertex(Vertex::new( p7, normal, uv));
        let i4 = mesh.add_vertex(Vertex::new( p3, normal, uv));
        // construct the 2 triangles, note the order of the vertices in the trigs
        mesh.add_triangle(i3,i2,i1);
        mesh.add_triangle(i4, i3, i1);

        // add the 2 right triangles
        let normal = Vec3::new(1.0,0.0,0.0);
        let i1 = mesh.add_vertex(Vertex::new( p4, normal, uv));
        let i2 = mesh.add_vertex(Vertex::new( p8, normal, uv));
        let i3 = mesh.add_vertex(Vertex::new( p7, normal, uv));
        let i4 = mesh.add_vertex(Vertex::new( p3, normal, uv));
        // construct the 2 triangles, note the order of the vertices in the trigs
        mesh.add_triangle(i1,i2,i3);
        mesh.add_triangle(i1, i3, i4);

        // add the 2 left triangles
        let normal = Vec3::new(-1.0,0.0,0.0);
        let i1 = mesh.add_vertex(Vertex::new( p1, normal, uv));
        let i2 = mesh.add_vertex(Vertex::new( p5, normal, uv));
        let i3 = mesh.add_vertex(Vertex::new( p6, normal, uv));
        let i4 = mesh.add_vertex(Vertex::new( p2, normal, uv));
        // construct the 2 triangles, note the order of the vertices in the trigs
        mesh.add_triangle(i3,i2,i1);
        mesh.add_triangle(i4, i3, i1);
          
    }
}