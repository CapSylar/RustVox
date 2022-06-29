use glam::{Vec3, Vec2, const_vec2};

use super::{mesh::{Vertex, Mesh}, types::Vec3i32};

const VOXEL_SIZE: f32 = 1.0;

#[derive(Clone,Copy)]
pub enum VoxelType
{
    Grass,
    Sand,
}

/// Holds the texture UV information for each block type
/// Each entry holds the lower left UV coordinates of the texture for the face
#[derive(Clone, Copy)]
struct VoxelTypeTexture
{
    top_face: Vec2,
    bottom_face: Vec2,
    front_face: Vec2,
    back_face: Vec2,
    right_face: Vec2,
    left_face: Vec2,
}

const VoxelUVData : [VoxelTypeTexture ; 2] = [
    VoxelTypeTexture{ // Grass
    top_face: const_vec2!([0.1,0.9]), back_face: const_vec2!([0.0,0.9]),
    bottom_face: const_vec2!([0.2,0.9]), front_face: const_vec2!([0.0,0.9]),
    left_face: const_vec2!([0.0,0.9]), right_face: const_vec2!([0.0,0.9])},
    VoxelTypeTexture{ // Sand
    top_face: const_vec2!([0.3,0.9]), back_face: const_vec2!([0.3,0.9]),
    bottom_face: const_vec2!([0.3,0.9]), front_face: const_vec2!([0.3,0.9]),
    left_face: const_vec2!([0.3,0.9]), right_face: const_vec2!([0.3,0.9])}
    ];

#[derive(Clone,Copy)]
pub enum VoxelFace
{
    TOP(Vec3i32),
    BOTTOM(Vec3i32), // if a cube is Axis aligned with the right hand coordinate system, position the compas on the top cube face to determine what face is north,east,south and west
    NORTH(Vec3i32),
    SOUTH(Vec3i32),
    EAST(Vec3i32),
    WEST(Vec3i32),
}

pub const VoxelFaceValues : [Vec3i32;6] = 
[
     Vec3i32::new(0, 1, 0),
     Vec3i32::new(0,-1,0),
     Vec3i32::new(0,0,1),
     Vec3i32::new(0,0,-1),
     Vec3i32::new(1,0,0),
     Vec3i32::new(-1,0,0)
];

#[derive(Clone,Copy)]
pub struct Voxel
{
    voxel_type : VoxelType,
    is_filled: bool,
}

impl Voxel
{
    pub fn new(voxel_type: VoxelType, is_filled: bool) -> Voxel
    {
        Voxel{voxel_type, is_filled}
    }

    pub fn new_default() -> Voxel
    {
        Voxel { voxel_type: VoxelType::Grass, is_filled: true }
    }

    pub fn set_filled(&mut self, filled: bool) { self.is_filled = filled; }

    pub fn is_filled(&self) -> bool { self.is_filled }

    pub fn set_type(&mut self, voxel_type : VoxelType ) {self.voxel_type = voxel_type }

    pub fn append_mesh_faces( &self, faces: &[bool;6], pos: Vec3 , mesh: &mut Mesh)
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

        let uv = VoxelUVData[self.voxel_type as usize];

        let uv1 = Vec2::new(0.0 ,0.0);
        let uv2 = Vec2::new(0.0 ,0.1);
        let uv3 = Vec2::new(0.1 ,0.1);
        let uv4 = Vec2::new(0.1 ,0.0);

        //TODO: could refactor by defining the vertices for each face, and then iterate

        if  faces[0] 
        {
            // add the 2 top triangles
            let normal = Vec3::new(0.0,1.0,0.0);
        
            let i1 = mesh.add_vertex(Vertex::new( p5, normal, uv.top_face + uv1));
            let i2 = mesh.add_vertex(Vertex::new( p6, normal, uv.top_face + uv2));
            let i3 = mesh.add_vertex(Vertex::new( p7, normal, uv.top_face + uv3));
            let i4 = mesh.add_vertex(Vertex::new( p8, normal, uv.top_face + uv4));
            // construct the 2 triangles
            mesh.add_triangle(i1,i2,i3);
            mesh.add_triangle(i1, i3, i4);
        }

        if faces[1]
        {
            // add the 2 bottom triangles
            let normal = Vec3::new(0.0,-1.0,0.0);
            let i1 = mesh.add_vertex(Vertex::new( p1, normal, uv.bottom_face + uv1));
            let i2 = mesh.add_vertex(Vertex::new( p2, normal, uv.bottom_face + uv2));
            let i3 = mesh.add_vertex(Vertex::new( p3, normal, uv.bottom_face + uv3));
            let i4 = mesh.add_vertex(Vertex::new( p4, normal, uv.bottom_face + uv4));
            // construct the 2 triangles, note the order of the vertices in the trigs
            mesh.add_triangle(i3,i2,i1);
            mesh.add_triangle(i4, i3, i1);
        }
        
        if faces[2]
        {
            // add the 2 front triangles
            let normal = Vec3::new(0.0,0.0,1.0);
            let i1 = mesh.add_vertex(Vertex::new( p1, normal, uv.front_face + uv1));
            let i2 = mesh.add_vertex(Vertex::new( p5, normal, uv.front_face + uv2));
            let i3 = mesh.add_vertex(Vertex::new( p8, normal, uv.front_face + uv3));
            let i4 = mesh.add_vertex(Vertex::new( p4, normal, uv.front_face + uv4));
            // construct the 2 triangles, note the order of the vertices in the trigs
            mesh.add_triangle(i1,i2,i3);
            mesh.add_triangle(i1, i3, i4);
        }

        if faces[3]
        {
            // add the 2 back triangles
            let normal = Vec3::new(0.0,0.0,-1.0);
            let i1 = mesh.add_vertex(Vertex::new( p2, normal, uv.back_face + uv1));
            let i2 = mesh.add_vertex(Vertex::new( p6, normal, uv.back_face + uv2));
            let i3 = mesh.add_vertex(Vertex::new( p7, normal, uv.back_face + uv3));
            let i4 = mesh.add_vertex(Vertex::new( p3, normal, uv.back_face + uv4));
            // construct the 2 triangles, note the order of the vertices in the trigs
            mesh.add_triangle(i3,i2,i1);
            mesh.add_triangle(i4, i3, i1);
        }

        if faces[4]
        {
            // add the 2 right triangles
            let normal = Vec3::new(1.0,0.0,0.0);
            let i1 = mesh.add_vertex(Vertex::new( p4, normal, uv.right_face + uv1));
            let i2 = mesh.add_vertex(Vertex::new( p8, normal, uv.right_face + uv2));
            let i3 = mesh.add_vertex(Vertex::new( p7, normal, uv.right_face + uv3));
            let i4 = mesh.add_vertex(Vertex::new( p3, normal, uv.right_face + uv4));
            // construct the 2 triangles, note the order of the vertices in the trigs
            mesh.add_triangle(i1,i2,i3);
            mesh.add_triangle(i1, i3, i4);
        }

        if faces[5]
        {
            // add the 2 left triangles
            let normal = Vec3::new(-1.0,0.0,0.0);
            let i1 = mesh.add_vertex(Vertex::new( p1, normal, uv.left_face + uv1));
            let i2 = mesh.add_vertex(Vertex::new( p5, normal, uv.left_face + uv2));
            let i3 = mesh.add_vertex(Vertex::new( p6, normal, uv.left_face + uv3));
            let i4 = mesh.add_vertex(Vertex::new( p2, normal, uv.left_face + uv4));
            // construct the 2 triangles, note the order of the vertices in the trigs
            mesh.add_triangle(i3,i2,i1);
            mesh.add_triangle(i4, i3, i1);
        }

    }

    /// Returns the Voxel's mesh for rendering
    pub fn append_mesh( &self ,pos: Vec3 ,mesh: &mut Mesh)
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

        let side_uv = Vec2::new(0.0,0.9);
        let top_uv = Vec2::new(0.1,0.9);
        let bottom_uv = Vec2::new(0.2,0.9);

        let eps: f32 = 0.001;

        let uv1 = Vec2::new(0.0 ,0.0);
        let uv2 = Vec2::new(0.0 ,0.1);
        let uv3 = Vec2::new(0.1 ,0.1);
        let uv4 = Vec2::new(0.1 ,0.0);

        // add the 2 top triangles
        let normal = Vec3::new(0.0,1.0,0.0);
        
        let i1 = mesh.add_vertex(Vertex::new( p5, normal, top_uv + uv1));
        let i2 = mesh.add_vertex(Vertex::new( p6, normal, top_uv + uv2));
        let i3 = mesh.add_vertex(Vertex::new( p7, normal, top_uv + uv3));
        let i4 = mesh.add_vertex(Vertex::new( p8, normal, top_uv + uv4));
        // construct the 2 triangles
        mesh.add_triangle(i1,i2,i3);
        mesh.add_triangle(i1, i3, i4);

        // add the 2 bottom triangles
        let normal = Vec3::new(0.0,-1.0,0.0);
        let i1 = mesh.add_vertex(Vertex::new( p1, normal, bottom_uv + uv1));
        let i2 = mesh.add_vertex(Vertex::new( p2, normal, bottom_uv + uv2));
        let i3 = mesh.add_vertex(Vertex::new( p3, normal, bottom_uv + uv3));
        let i4 = mesh.add_vertex(Vertex::new( p4, normal, bottom_uv + uv4));
        // construct the 2 triangles, note the order of the vertices in the trigs
        mesh.add_triangle(i3,i2,i1);
        mesh.add_triangle(i4, i3, i1);

        // add the 2 front triangles
        let normal = Vec3::new(0.0,0.0,1.0);
        let i1 = mesh.add_vertex(Vertex::new( p1, normal, side_uv + uv1));
        let i2 = mesh.add_vertex(Vertex::new( p5, normal, side_uv + uv2));
        let i3 = mesh.add_vertex(Vertex::new( p8, normal, side_uv + uv3));
        let i4 = mesh.add_vertex(Vertex::new( p4, normal, side_uv + uv4));
        // construct the 2 triangles, note the order of the vertices in the trigs
        mesh.add_triangle(i1,i2,i3);
        mesh.add_triangle(i1, i3, i4);

        // add the 2 back triangles
        let normal = Vec3::new(0.0,0.0,-1.0);
        let i1 = mesh.add_vertex(Vertex::new( p2, normal, side_uv + uv1));
        let i2 = mesh.add_vertex(Vertex::new( p6, normal, side_uv + uv2));
        let i3 = mesh.add_vertex(Vertex::new( p7, normal, side_uv + uv3));
        let i4 = mesh.add_vertex(Vertex::new( p3, normal, side_uv + uv4));
        // construct the 2 triangles, note the order of the vertices in the trigs
        mesh.add_triangle(i3,i2,i1);
        mesh.add_triangle(i4, i3, i1);

        // add the 2 right triangles
        let normal = Vec3::new(1.0,0.0,0.0);
        let i1 = mesh.add_vertex(Vertex::new( p4, normal, side_uv + uv1));
        let i2 = mesh.add_vertex(Vertex::new( p8, normal, side_uv + uv2));
        let i3 = mesh.add_vertex(Vertex::new( p7, normal, side_uv + uv3));
        let i4 = mesh.add_vertex(Vertex::new( p3, normal, side_uv + uv4));
        // construct the 2 triangles, note the order of the vertices in the trigs
        mesh.add_triangle(i1,i2,i3);
        mesh.add_triangle(i1, i3, i4);

        // add the 2 left triangles
        let normal = Vec3::new(-1.0,0.0,0.0);
        let i1 = mesh.add_vertex(Vertex::new( p1, normal, side_uv + uv1));
        let i2 = mesh.add_vertex(Vertex::new( p5, normal, side_uv + uv2));
        let i3 = mesh.add_vertex(Vertex::new( p6, normal, side_uv + uv3));
        let i4 = mesh.add_vertex(Vertex::new( p2, normal, side_uv + uv4));
        // construct the 2 triangles, note the order of the vertices in the trigs
        mesh.add_triangle(i3,i2,i1);
        mesh.add_triangle(i4, i3, i1);
          
    }
}