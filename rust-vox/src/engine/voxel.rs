use glam::{Vec3, Vec2, const_vec2};

use super::{geometry::{mesh::{Mesh}, opengl_vertex::OpenglVertex}, renderer::opengl_abstractions::vertex_array::VertexLayout};

const VOXEL_SIZE: f32 = 1.0;

// Voxel Vertex
/// Holds all the data which constitute a *vertex*
#[repr(C,packed)]
#[derive(Clone,Copy,Debug)]
pub struct VoxelVertex
{
    position: Vec3, // X, Y, Z
    tex_coord: Vec2, // U, V
    normal_index : u8, // byte index into a normal LUT in the shader, 6 possible normal vectors
}

impl VoxelVertex
{
    pub fn new( position: Vec3 , normal_index : u8 , texture_coordinate: Vec2 ) -> Self
    {
        Self { position , tex_coord: texture_coordinate, normal_index  }
    }
}

impl OpenglVertex for VoxelVertex
{
    fn get_layout() -> VertexLayout
    {
        let mut vertex_layout = VertexLayout::new();

        vertex_layout.push_f32(3); // vertex(x,y,z)
        vertex_layout.push_f32(2); // uv coordinates(u,v)
        vertex_layout.push_u8(1); // Normal Index

        vertex_layout
    }
}

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
    top_face: const_vec2!([0.125,0.875]), back_face: const_vec2!([0.0,0.875]),
    bottom_face: const_vec2!([0.25,0.875]), front_face: const_vec2!([0.0,0.875]),
    left_face: const_vec2!([0.0,0.875]), right_face: const_vec2!([0.0,0.875])},
    VoxelTypeTexture{ // Sand
    top_face: const_vec2!([0.375,0.875]), back_face: const_vec2!([0.375,0.875]),
    bottom_face: const_vec2!([0.375,0.875]), front_face: const_vec2!([0.375,0.875]),
    left_face: const_vec2!([0.375,0.875]), right_face: const_vec2!([0.375,0.875])}
    ];

// #[derive(Clone,Copy)]
// pub enum VoxelFace
// {
//     TOP(IVec3),
//     BOTTOM(IVec3), // if a cube is Axis aligned with the right hand coordinate system, position the compas on the top cube face to determine what face is north,east,south and west
//     NORTH(IVec3),
//     SOUTH(IVec3),
//     EAST(IVec3),
//     WEST(IVec3),
// }

pub const VOXEL_FACE_VALUES : [(i32,i32,i32);6] = 
[
    (0,1,0),
    (0,-1,0),
    (0,0,1),
    (0,0,-1),
    (1,0,0),
    (-1,0,0)
];

pub enum Normals
{
    POSY,NEGY,POSZ,NEGZ,POSX,NEGX
}

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

    pub fn append_mesh_faces( &self, faces: &[bool;6], pos: Vec3 , mesh: &mut Mesh<VoxelVertex>)
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
        let uv2 = Vec2::new(0.0 ,0.125);
        let uv3 = Vec2::new(0.125 ,0.125);
        let uv4 = Vec2::new(0.125 ,0.0);

        //TODO: could refactor by defining the vertices for each face, and then iterate

       if faces[0] 
        {
            // add the 2 top triangles
            let i1 = mesh.add_vertex(VoxelVertex::new( p5, Normals::POSY as u8, uv.top_face + uv1));
            let i2 = mesh.add_vertex(VoxelVertex::new( p6, Normals::POSY as u8, uv.top_face + uv2));
            let i3 = mesh.add_vertex(VoxelVertex::new( p7, Normals::POSY as u8, uv.top_face + uv3));
            let i4 = mesh.add_vertex(VoxelVertex::new( p8, Normals::POSY as u8, uv.top_face + uv4));
            // construct the 2 triangles
            mesh.add_triangle(i1,i2,i3);
            mesh.add_triangle(i1, i3, i4);
        }

        if faces[1]
        {
            // add the 2 bottom triangles
            let i1 = mesh.add_vertex(VoxelVertex::new( p1, Normals::NEGY as u8, uv.bottom_face + uv1));
            let i2 = mesh.add_vertex(VoxelVertex::new( p2, Normals::NEGY as u8, uv.bottom_face + uv2));
            let i3 = mesh.add_vertex(VoxelVertex::new( p3, Normals::NEGY as u8, uv.bottom_face + uv3));
            let i4 = mesh.add_vertex(VoxelVertex::new( p4, Normals::NEGY as u8, uv.bottom_face + uv4));
            // construct the 2 triangles, note the order of the vertices in the trigs
            mesh.add_triangle(i3,i2,i1);
            mesh.add_triangle(i4, i3, i1);
        }
        
        if faces[2]
        {
            // add the 2 front triangles
            let i1 = mesh.add_vertex(VoxelVertex::new( p1, Normals::POSZ as u8, uv.front_face + uv1));
            let i2 = mesh.add_vertex(VoxelVertex::new( p5, Normals::POSZ as u8, uv.front_face + uv2));
            let i3 = mesh.add_vertex(VoxelVertex::new( p8, Normals::POSZ as u8, uv.front_face + uv3));
            let i4 = mesh.add_vertex(VoxelVertex::new( p4, Normals::POSZ as u8, uv.front_face + uv4));
            // construct the 2 triangles, note the order of the vertices in the trigs
            mesh.add_triangle(i1,i2,i3);
            mesh.add_triangle(i1, i3, i4);
        }

        if faces[3]
        {
            // add the 2 back triangles
            let i1 = mesh.add_vertex(VoxelVertex::new( p2, Normals::NEGZ as u8, uv.back_face + uv1));
            let i2 = mesh.add_vertex(VoxelVertex::new( p6, Normals::NEGZ as u8, uv.back_face + uv2));
            let i3 = mesh.add_vertex(VoxelVertex::new( p7, Normals::NEGZ as u8, uv.back_face + uv3));
            let i4 = mesh.add_vertex(VoxelVertex::new( p3, Normals::NEGZ as u8, uv.back_face + uv4));
            // construct the 2 triangles, note the order of the vertices in the trigs
            mesh.add_triangle(i3,i2,i1);
            mesh.add_triangle(i4, i3, i1);
        }

        if faces[4]
        {
            // add the 2 right triangles
            let i1 = mesh.add_vertex(VoxelVertex::new( p4, Normals::POSX as u8, uv.right_face + uv1));
            let i2 = mesh.add_vertex(VoxelVertex::new( p8, Normals::POSX as u8, uv.right_face + uv2));
            let i3 = mesh.add_vertex(VoxelVertex::new( p7, Normals::POSX as u8, uv.right_face + uv3));
            let i4 = mesh.add_vertex(VoxelVertex::new( p3, Normals::POSX as u8, uv.right_face + uv4));
            // construct the 2 triangles, note the order of the vertices in the trigs
            mesh.add_triangle(i1,i2,i3);
            mesh.add_triangle(i1, i3, i4);
        }

        if faces[5]
        {
            // add the 2 left triangles
            let i1 = mesh.add_vertex(VoxelVertex::new( p1, Normals::NEGX as u8, uv.left_face + uv1));
            let i2 = mesh.add_vertex(VoxelVertex::new( p5, Normals::NEGX as u8, uv.left_face + uv2));
            let i3 = mesh.add_vertex(VoxelVertex::new( p6, Normals::NEGX as u8, uv.left_face + uv3));
            let i4 = mesh.add_vertex(VoxelVertex::new( p2, Normals::NEGX as u8, uv.left_face + uv4));
            // construct the 2 triangles, note the order of the vertices in the trigs
            mesh.add_triangle(i3,i2,i1);
            mesh.add_triangle(i4, i3, i1);
        }

    }
}