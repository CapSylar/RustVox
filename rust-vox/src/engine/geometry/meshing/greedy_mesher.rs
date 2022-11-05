use std::{thread::current, num::NonZeroI128, string, f32::consts::E, mem::swap};

use glam::{Vec2, Vec3, IVec3};
use crate::engine::{chunk::{Chunk, CHUNK_SIZE}, geometry::{voxel_vertex::VoxelVertex, mesh::Mesh, opengl_vertex::OpenglVertex, voxel::{VoxelType, Voxel, self}}};
use super::chunk_mesher::{ChunkMesher, VOXEL_SIZE, UVs};

pub struct GreedyMesher;

impl GreedyMesher
{
    //TODO: this is a mess
    fn add_quad(mesh: &mut Mesh<VoxelVertex>, face:(u8,VoxelType), x_dir: usize , y_dir: usize, lower_left:Vec3, upper_left: Vec3, upper_right:Vec3, lower_right:Vec3)
    {
        let texture_index = face.1 as u8;

        let lower_left_uv: (u8,u8);
        let upper_left_uv: (u8,u8);
        let upper_right_uv: (u8,u8);
        let lower_right_uv: (u8,u8);

        let x = (lower_right[x_dir] - lower_left[x_dir]).abs() as u8;
        let y = (upper_right[y_dir] - lower_right[y_dir]).abs() as u8;

        if y_dir == 0 // doing a pass in the +Z direction
        {
            lower_left_uv = (y,0);
            upper_left_uv = (0,0);
            lower_right_uv = (y,x);
            upper_right_uv = (0,x);
        }
        else
        {
            lower_left_uv = (0,0);
            upper_left_uv = (0,y);
            lower_right_uv = (x,0);
            upper_right_uv = (x,y);
        }

        let lower_left = VoxelVertex::new(lower_left * VOXEL_SIZE,0,lower_left_uv, texture_index);
        let upper_left =  VoxelVertex::new(upper_left * VOXEL_SIZE,0,upper_left_uv, texture_index);
        let upper_right = VoxelVertex::new(upper_right * VOXEL_SIZE,0,upper_right_uv, texture_index);
        let lower_right = VoxelVertex::new(lower_right * VOXEL_SIZE,0,lower_right_uv, texture_index);

        if face.0 == 1
        {
            mesh.add_quad(lower_left, upper_left, upper_right, lower_right);
        }
        else // in the opposite direction
        {
            mesh.add_quad(lower_right, upper_right, upper_left, lower_left);
        }
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

            let mut mask = vec![(0u8,VoxelType::Dirt);CHUNK_SIZE[n_dir] * CHUNK_SIZE[nn_dir]];
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

                        let current_opaque = 
                            if slice >=0
                            {
                                let chunk = chunk.get_voxel(current_pos[0] as i32, current_pos[1] as i32, current_pos[2] as i32).unwrap();
                                (chunk.is_filled(),chunk.voxel_type)
                            }
                            else {(false,VoxelType::Dirt)} ;
                        
                        let next_opaque = if slice < (CHUNK_SIZE[current_dir] as i32 -1)
                        {
                            let chunk = chunk.get_voxel(current_pos[0] as i32 + offset[0], current_pos[1] as i32 + offset[1], current_pos[2] as i32 + offset[2]).unwrap();
                            (chunk.is_filled(),chunk.voxel_type)
                        }
                        else {(false,VoxelType::Dirt)} ;
                        
                        if current_opaque.0 == next_opaque.0
                        {
                            mask[mask_index].0 = 0;
                        }
                        else if !current_opaque.0 && next_opaque.0
                        {
                            mask[mask_index] = (1,next_opaque.1); // quad is facing us in the current direction
                        }
                        else
                        {
                            mask[mask_index] = (2,current_opaque.1); // quad is facing the opposite direction
                        }

                        mask_index += 1;
                    }
                }
                
                current_pos[current_dir] += 1; // TODO: Document

                // Step 2: use the mask and iterate over every block in this slice of the chunk
                // print the mask

                // iterate over the faces of the slice
                let mut mask_index = 0;
                for j in 0..CHUNK_SIZE[n_dir]
                {
                    let mut i = 0;
                    while i < CHUNK_SIZE[nn_dir]
                    {
                        if mask[mask_index].0 != 0 // if current face is visible
                        {
                            let face_direction = mask[mask_index];
                            // search along the current axis until mask[mask_index + w] is false, we are searching the quad with height 1 and the largest possible width

                            let mut width = 1;
                            while (i + width) < CHUNK_SIZE[nn_dir] && mask[mask_index+width] == face_direction // they must also be of the same type
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

                            // current_pos refers to the lower left vertex in the quad (assuming a front facing quad that is visible)
                            current_pos[n_dir] = j as i32;
                            current_pos[nn_dir] = i as i32;

                            let mut du: [i32;3] = [0,0,0];
                            du[nn_dir] = width as i32;

                            let mut dv: [i32;3] = [0,0,0];
                            dv[n_dir] = height as i32;

                            // append the quad
                            let upper_left = IVec3::new(current_pos[0] + dv[0],current_pos[1] + dv[1],current_pos[2] + dv[2]).as_vec3() + chunk.pos_world_space();
                            let lower_left = IVec3::new(current_pos[0],current_pos[1],current_pos[2]).as_vec3() + chunk.pos_world_space();
                            let lower_right = IVec3::new(current_pos[0] + du[0],current_pos[1] + du[1],current_pos[2] + du[2]).as_vec3() + chunk.pos_world_space();
                            let upper_right = IVec3::new(current_pos[0] + du[0] + dv[0],current_pos[1] + du[1] + dv[1],current_pos[2] + du[2] + dv[2]).as_vec3() + chunk.pos_world_space();

                            GreedyMesher::add_quad(&mut mesh, face_direction, nn_dir, n_dir, lower_left, upper_left, upper_right, lower_right);

                            // clear the mask for each face that was used
                            for w in 0..width
                            {
                                for h in 0..height
                                {
                                    mask[mask_index + w + h * CHUNK_SIZE[nn_dir]].0 = 0; // careful
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