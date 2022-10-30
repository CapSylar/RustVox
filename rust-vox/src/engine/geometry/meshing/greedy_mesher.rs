use std::thread::current;

use glam::{Vec2, Vec3};
use crate::engine::{chunk::{Chunk, CHUNK_SIZE}, geometry::{voxel_vertex::VoxelVertex, mesh::Mesh, opengl_vertex::OpenglVertex}};
use super::chunk_mesher::{ChunkMesher, VOXEL_SIZE};

pub struct GreedyMesher;

impl GreedyMesher
{
    //TODO: this is a mess
    fn add_quad(mesh: &mut Mesh<VoxelVertex>, lower_left:(i32,i32,i32), upper_left: (i32,i32,i32), upper_right:(i32,i32,i32), lower_right:(i32,i32,i32))
    {
        // append the quad
        mesh.add_quad(
            VoxelVertex::new(Vec3::new(lower_left.0 as f32 * VOXEL_SIZE,lower_left.1 as f32 * VOXEL_SIZE,lower_left.2 as f32 * VOXEL_SIZE),0,Vec2::ZERO),
            VoxelVertex::new(Vec3::new(upper_left.0 as f32 * VOXEL_SIZE, upper_left.1 as f32 * VOXEL_SIZE, upper_left.2 as f32 * VOXEL_SIZE),0,Vec2::ZERO),
            VoxelVertex::new(Vec3::new(upper_right.0 as f32 * VOXEL_SIZE,upper_right.1 as f32 * VOXEL_SIZE,upper_right.2 as f32 * VOXEL_SIZE),0,Vec2::ZERO),
            VoxelVertex::new(Vec3::new(lower_right.0 as f32 * VOXEL_SIZE, lower_right.1 as f32 * VOXEL_SIZE, lower_right.2 as f32 * VOXEL_SIZE),0,Vec2::ZERO)
        );
    }
}

impl ChunkMesher for GreedyMesher
{
    fn generate_mesh(chunk: &Chunk) -> Mesh<VoxelVertex>
    {
        let mut mesh = Mesh::new();
        // sweep over each axis separately (X,Y,Z)

        for current_dir in 0usize..3 // 0 is X, 1 is Y, 2 is Z
        {
            let mut current_pos: [i32;3] = [0,0,0]; // X,Y,Z

            let n_dir = (current_dir+1) % 3;
            let nn_dir = (current_dir+2) % 3;

            //TODO: better documentation
            // mask coverting every voxel in the direction we are traversing, as if we took a knife and cut through the chunk perpendicular to the direction
            // we are traversing, every voxel in the cut has an entry

            let mut mask = vec![0u8;CHUNK_SIZE[n_dir] * CHUNK_SIZE[nn_dir]];
            let mut offset : [i32;3] = [0,0,0];
            offset[current_dir] = 1;

            // check each slice of the chunk one at a time
            for slice in -1..CHUNK_SIZE[current_dir] as i32
            {
                // Step 1: populate the mask for the current slice
                let mut mask_index: usize = 0;
                current_pos[current_dir] = slice;

                for mask_y in 0..CHUNK_SIZE[n_dir] as i32
                {
                    current_pos[n_dir] = mask_y;
                    for mask_x in 0..CHUNK_SIZE[nn_dir] as i32
                    {
                        current_pos[nn_dir] = mask_x;
                        let current_opaque = if slice >=0 {chunk.get_voxel(current_pos[0] as i32, current_pos[1] as i32, current_pos[2] as i32).unwrap().is_filled()} else {false} ;
                        let next_opaque = if slice < (CHUNK_SIZE[current_dir] as i32 -1) {chunk.get_voxel(current_pos[0] as i32 + offset[0], current_pos[1] as i32 + offset[1], current_pos[2] as i32 + offset[2]).unwrap().is_filled()} else {false} ;
                        
                        if current_opaque == next_opaque
                        {
                            mask[mask_index] = 0;
                        }
                        else if !current_opaque && next_opaque
                        {
                            mask[mask_index] = 1; // face in the current direction
                        }
                        else
                        {
                            mask[mask_index] = 2; // face in the opposite direction
                        }

                        mask_index += 1;
                    }
                }
                
                current_pos[current_dir] += 1; // TODO: Document

                // Step 2: use the mask and iterate over every block in this slice of the chunk
                
                // iterate over the faces of the slice
                let mut mask_index = 0;
                for j in 0..CHUNK_SIZE[n_dir]
                {
                    let mut i = 0;
                    while i < CHUNK_SIZE[nn_dir]
                    {
                        if mask[mask_index] != 0 // if current face is visible
                        {
                            let face_direction = mask[mask_index];
                            // search along the current axis until mask[mask_index + w] is false, we are searching the quad with height 1 and the largest possible width

                            let mut width = 1;
                            while (i + width) < CHUNK_SIZE[nn_dir] && mask[mask_index+width] == face_direction
                            {
                                width += 1;
                            }

                            // we have the biggest width, compure the biggest height that we can have while still maintaining the current width
                            // there should be no holes in the resulting quad generated
                            let mut height = 1;
                    'outer: while height + j < CHUNK_SIZE[n_dir]
                            {
                                // for each height, loop over all the faces in the width making sure there are no holes
                                for w in 0..width
                                {
                                    if mask[mask_index + w + height * CHUNK_SIZE[nn_dir]] != face_direction // carefull
                                    {
                                        break 'outer;
                                    }
                                }
                                height += 1;
                            }

                            // at this point, the best width and height have been computed
                            // emit the quad

                            // current_pos refers to the top left vertex in the quad (assuming a front facing quad that is visible)
                            current_pos[n_dir] = j as i32;
                            current_pos[nn_dir] = i as i32;

                            let mut du: [i32;3] = [0,0,0];
                            du[nn_dir] = width as i32;

                            let mut dv: [i32;3] = [0,0,0];
                            dv[n_dir] = height as i32;

                            // append the quad
                            let upper_left = (current_pos[0] + dv[0],current_pos[1] + dv[1],current_pos[2] + dv[2]);
                            let lower_left = (current_pos[0],current_pos[1],current_pos[2]);
                            let lower_right = (current_pos[0] + du[0],current_pos[1] + du[1],current_pos[2] + du[2]);
                            let upper_right = (current_pos[0] + du[0] + dv[0],current_pos[1] + du[1] + dv[1],current_pos[2] + du[2] + dv[2]);

                            if face_direction == 1 // along the direction
                            {
                                GreedyMesher::add_quad(&mut mesh, lower_left, upper_left, upper_right, lower_right);
                            }
                            else // in the opposite direction, simply reverse the vertices
                            {
                                GreedyMesher::add_quad(&mut mesh, lower_right, upper_right, upper_left, lower_left);
                            }

                            // clear the mask for each face that was used
                            for w in 0..width
                            {
                                for h in 0..height
                                {
                                    mask[mask_index + w + h * CHUNK_SIZE[nn_dir]] = 0; // carefull
                                }
                            }

                            // increment counters
                            mask_index += width;
                            i += width;
                        }
                        else // mask is false
                        {   
                            mask_index += 1;
                            i += 1;
                        }
                    }
                }
            }
        }
        mesh
    }
}