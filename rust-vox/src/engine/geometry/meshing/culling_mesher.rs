use glam::{Vec3, IVec3};

use crate::engine::{geometry::{voxel::{Voxel, VoxelType}, voxel_vertex::VoxelVertex, mesh::Mesh}, chunk::{Chunk, CHUNK_SIZE_X, CHUNK_SIZE_Y, CHUNK_SIZE_Z, Face}};

use super::chunk_mesher::{Direction, ChunkMesher, VOXEL_SIZE, VOXEL_FACE_VALUES};

pub struct CullingMesher;

impl CullingMesher
{
    fn append_voxel_mesh_faces(voxel: Voxel, faces: &[bool;6], pos: Vec3 , mesh: &mut Mesh<VoxelVertex>)
    {
        // generate the 8 vertices to draw the voxel
        //bottom
        let p1 = Vec3::new(pos.x + 0.0,pos.y + 0.0,pos.z + VOXEL_SIZE);
        let p2 = Vec3::new(pos.x + 0.0,pos.y + 0.0,pos.z);
        let p3 = Vec3::new(pos.x + VOXEL_SIZE,pos.y + 0.0,pos.z);
        let p4 = Vec3::new(pos.x + VOXEL_SIZE,pos.y + 0.0,pos.z + VOXEL_SIZE);

        //top
        let p5 = Vec3::new(pos.x + 0.0,pos.y + VOXEL_SIZE,pos.z + VOXEL_SIZE);
        let p6 = Vec3::new(pos.x + 0.0,pos.y + VOXEL_SIZE,pos.z);
        let p7 = Vec3::new(pos.x + VOXEL_SIZE,pos.y + VOXEL_SIZE,pos.z);
        let p8 = Vec3::new(pos.x + VOXEL_SIZE,pos.y + VOXEL_SIZE,pos.z + VOXEL_SIZE);

       if faces[0] 
        {
            // add the 2 top triangles
            mesh.add_quad(
                VoxelVertex::new( p5, Direction::Posy,(0,0), voxel),
                VoxelVertex::new( p6, Direction::Posy, (0,1), voxel),
                VoxelVertex::new( p7, Direction::Posy, (1,1), voxel),
                VoxelVertex::new( p8, Direction::Posy, (1,0), voxel)
            );
        }

        if faces[1]
        {
            // add the 2 bottom triangles
            mesh.add_quad(
                VoxelVertex::new( p3, Direction::Negy, (0,0), voxel),
                VoxelVertex::new( p2, Direction::Negy, (0,1), voxel),
                VoxelVertex::new( p1, Direction::Negy, (1,1), voxel),
                VoxelVertex::new( p4, Direction::Negy, (1,0), voxel)
            );
        }
        
        if faces[2]
        {
            // add the 2 front triangles
            mesh.add_quad(
                VoxelVertex::new( p1, Direction::Posz, (0,0), voxel),
                VoxelVertex::new( p5, Direction::Posz, (0,1), voxel),
                VoxelVertex::new( p8, Direction::Posz, (1,1), voxel),
                VoxelVertex::new( p4, Direction::Posz, (1,0), voxel)
            );
        }

        if faces[3]
        {
            // add the 2 back triangles
            mesh.add_quad(
                VoxelVertex::new( p7, Direction::Negz, (0,0), voxel),
                VoxelVertex::new( p6, Direction::Negz, (0,1), voxel),
                VoxelVertex::new( p2, Direction::Negz, (1,1), voxel),
                VoxelVertex::new( p3, Direction::Negz, (1,0), voxel)
            );
        }

        if faces[4]
        {
            // add the 2 right triangles
            mesh.add_quad(
                VoxelVertex::new( p4, Direction::Posx, (0,0), voxel),
                VoxelVertex::new( p8, Direction::Posx, (0,1), voxel),
                VoxelVertex::new( p7, Direction::Posx, (1,1), voxel),
                VoxelVertex::new( p3, Direction::Posx, (1,0), voxel)
            );
        }

        if faces[5]
        {
            // add the 2 left triangles
            mesh.add_quad(
                VoxelVertex::new( p6, Direction::Negx, (0,0), voxel),
                VoxelVertex::new( p5, Direction::Negx, (0,1), voxel),
                VoxelVertex::new( p1, Direction::Negx, (1,1), voxel),
                VoxelVertex::new( p2, Direction::Negx, (1,0), voxel)
            );
        }

    }
}

impl ChunkMesher for CullingMesher
{
    fn generate_mesh(chunk: &Chunk, mesh: &mut Mesh<VoxelVertex>, trans_faces: &mut Vec<Face>)
    {        
        //Generate the directly in here, good enough for now
        // for now render the mesh of all the voxels as is
        for x in 0..CHUNK_SIZE_X
        {
            for y in 0..CHUNK_SIZE_Y
            {
                for z in 0..CHUNK_SIZE_Z
                {
                    if chunk.voxels[x][y][z].is_filled()
                    {
                        let mut faces_to_render: [bool;6] = [false;6];

                        // TODO: refactor
                        for (index, offset) in VOXEL_FACE_VALUES.iter().enumerate()
                        {
                            // is the neighbor of the current voxel in the given direction filled ? 
                            let pos = IVec3::new(x as i32 + offset.0,y as i32 + offset.1,z as i32 + offset.2);
                            if let Some(neighbor) = chunk.get_voxel(pos)
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

                        CullingMesher::append_voxel_mesh_faces(chunk.voxels[x][y][z],
                            &faces_to_render,
                            chunk.pos_world_space() + Vec3::new(x as f32,y as f32,z as f32),
                            mesh);
                    }
                }
            }
        }
    }
}