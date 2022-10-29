use glam::{IVec3, Vec3, Vec2, const_vec2};

use crate::engine::chunk::{CHUNK_X, CHUNK_Y, CHUNK_Z, Chunk};

use super::{mesh::Mesh, voxel::{VoxelType}, voxel_vertex::VoxelVertex};


pub trait ChunkMesher
{
    /// generate the mesh for the chunk
    fn generate_mesh(chunk: &Chunk) -> Mesh<VoxelVertex>;
}

const VOXEL_SIZE: f32 = 1.0;

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

const VOXEL_UVDATA : [VoxelTypeTexture ; 2] = [
    VoxelTypeTexture{ // Grass
    top_face: const_vec2!([0.125,0.875]), back_face: const_vec2!([0.0,0.875]),
    bottom_face: const_vec2!([0.25,0.875]), front_face: const_vec2!([0.0,0.875]),
    left_face: const_vec2!([0.0,0.875]), right_face: const_vec2!([0.0,0.875])},
    VoxelTypeTexture{ // Sand
    top_face: const_vec2!([0.375,0.875]), back_face: const_vec2!([0.375,0.875]),
    bottom_face: const_vec2!([0.375,0.875]), front_face: const_vec2!([0.375,0.875]),
    left_face: const_vec2!([0.375,0.875]), right_face: const_vec2!([0.375,0.875])}
    ];

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
    Posy,Negy,Posz,Negz,Posx,Negx
}

pub struct CullingMesher;

impl CullingMesher
{
    fn append_voxel_mesh_faces(voxel_type: VoxelType, faces: &[bool;6], pos: Vec3 , mesh: &mut Mesh<VoxelVertex>)
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

        let uv = VOXEL_UVDATA[voxel_type as usize];

        let uv1 = Vec2::new(0.0 ,0.0);
        let uv2 = Vec2::new(0.0 ,0.125);
        let uv3 = Vec2::new(0.125 ,0.125);
        let uv4 = Vec2::new(0.125 ,0.0);

        //TODO: could refactor by defining the vertices for each face, and then iterate

       if faces[0] 
        {
            // add the 2 top triangles
            mesh.add_quad(
                VoxelVertex::new( p5, Normals::Posy as u8,uv.top_face + uv1),
                VoxelVertex::new( p6, Normals::Posy as u8, uv.top_face + uv2),
                VoxelVertex::new( p7, Normals::Posy as u8, uv.top_face + uv3),
                VoxelVertex::new( p8, Normals::Posy as u8, uv.top_face + uv4)
            );
        }

        if faces[1]
        {
            // add the 2 bottom triangles
            mesh.add_quad(
                VoxelVertex::new( p3, Normals::Negy as u8, uv.bottom_face + uv3),
                VoxelVertex::new( p2, Normals::Negy as u8, uv.bottom_face + uv2),
                VoxelVertex::new( p1, Normals::Negy as u8, uv.bottom_face + uv1),
                VoxelVertex::new( p4, Normals::Negy as u8, uv.bottom_face + uv4)
            );
        }
        
        if faces[2]
        {
            // add the 2 front triangles
            mesh.add_quad(
                VoxelVertex::new( p1, Normals::Posz as u8, uv.front_face + uv1),
                VoxelVertex::new( p5, Normals::Posz as u8, uv.front_face + uv2),
                VoxelVertex::new( p8, Normals::Posz as u8, uv.front_face + uv3),
                VoxelVertex::new( p4, Normals::Posz as u8, uv.front_face + uv4)
            );
        }

        if faces[3]
        {
            // add the 2 back triangles
            mesh.add_quad(
                VoxelVertex::new( p7, Normals::Negz as u8, uv.back_face + uv3),
                VoxelVertex::new( p6, Normals::Negz as u8, uv.back_face + uv2),
                VoxelVertex::new( p2, Normals::Negz as u8, uv.back_face + uv1),
                VoxelVertex::new( p3, Normals::Negz as u8, uv.back_face + uv4)
            );
        }

        if faces[4]
        {
            // add the 2 right triangles
            mesh.add_quad(
                VoxelVertex::new( p4, Normals::Posx as u8, uv.right_face + uv1),
                VoxelVertex::new( p8, Normals::Posx as u8, uv.right_face + uv2),
                VoxelVertex::new( p7, Normals::Posx as u8, uv.right_face + uv3),
                VoxelVertex::new( p3, Normals::Posx as u8, uv.right_face + uv4)
            );
        }

        if faces[5]
        {
            // add the 2 left triangles
            mesh.add_quad(
                VoxelVertex::new( p6, Normals::Negx as u8, uv.left_face + uv3),
                VoxelVertex::new( p5, Normals::Negx as u8, uv.left_face + uv2),
                VoxelVertex::new( p1, Normals::Negx as u8, uv.left_face + uv1),
                VoxelVertex::new( p2, Normals::Negx as u8, uv.left_face + uv4)
            );
        }

    }
}

impl ChunkMesher for CullingMesher
{
    fn generate_mesh(chunk: &Chunk) -> Mesh<VoxelVertex>
    {        
        // Generate the mesh
        let mut mesh: Mesh<VoxelVertex> = Mesh::new();
        
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

                        CullingMesher::append_voxel_mesh_faces( chunk.voxels[x][y][z].voxel_type,
                            &faces_to_render,
                            chunk.pos_world_space() + Vec3::new(x as f32,y as f32,z as f32),
                            &mut mesh);
                    }
                }
            }
        }

        mesh
    }
}

struct GreedyMesher;

impl ChunkMesher for GreedyMesher
{
    fn generate_mesh(chunk: &Chunk) -> Mesh<VoxelVertex>
    {
        todo!()
    }
}