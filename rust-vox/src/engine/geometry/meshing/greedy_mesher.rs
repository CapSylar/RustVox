use glam::{Vec3, IVec3};
use crate::engine::{chunk::{CHUNK_SIZE, CHUNK_SIZE_Y, CHUNK_SIZE_X}, geometry::{voxel_vertex::VoxelVertex, mesh::Mesh, voxel::{Voxel,VoxelType}, chunk_mesh::Face}};
use super::{chunk_mesher::{ChunkMesher, VOXEL_SIZE, NormalDirection}, voxel_fetcher::VoxelFetcher};

pub struct GreedyMesher;

// TODO: Document
pub const FACE_POSITION: [Vec3; 3] = [
    Vec3::new(0.0,0.5,0.5), // walking along X
    Vec3::new(0.5,0.0,0.5), // walking along Y
    Vec3::new(0.5,0.5,0.0)]; // walking along Z

impl GreedyMesher
{
    //TODO: this is a mess
    // keeps the transparent faces in front
    fn add_quad(mesh: &mut Mesh<VoxelVertex>, trans_faces: &mut Vec<Face>, face_pos: Vec3, face: SliceFace, current_pass_dir: usize, x_dir: usize , y_dir: usize, lower_left:Vec3, upper_left: Vec3, upper_right:Vec3, lower_right:Vec3)
    {
        let mut normal_dir = NormalDirection::from_index(current_pass_dir);
        if face.face_state == FaceState::CurrentDirection {normal_dir = normal_dir.opposite();} // reverse direction if face is actually facing the opposite direction

        let lower_left_uv: (u8,u8);
        let upper_left_uv: (u8,u8);
        let upper_right_uv: (u8,u8);
        let lower_right_uv: (u8,u8);

        // get in the number of voxels that span in the U direction, same for the V direction
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

        let lower_left = VoxelVertex::new(lower_left * VOXEL_SIZE,normal_dir,lower_left_uv, face.voxel);
        let upper_left =  VoxelVertex::new(upper_left * VOXEL_SIZE,normal_dir,upper_left_uv, face.voxel);
        let upper_right = VoxelVertex::new(upper_right * VOXEL_SIZE,normal_dir,upper_right_uv, face.voxel);
        let lower_right = VoxelVertex::new(lower_right * VOXEL_SIZE,normal_dir,lower_right_uv, face.voxel);

        if face.face_state == FaceState::CurrentDirection
        {
            mesh.add_quad(lower_left, upper_left, upper_right, lower_right);
        }
        else // in the opposite direction
        {
            mesh.add_quad(lower_right, upper_right, upper_left, lower_left);
        }

        // if the face is transparent, add it to the transparent faces list
        if face.voxel.is_transparent()
        {          
            trans_faces.push(Face::new(face_pos, trans_faces.len() * 6));
            // make sure the transparent indices are grouped in front in the index list
            // swap the quad indices at index trans_faces.len() in the index buffer with the current inserted quad indices
            let dst = (trans_faces.len()-1) * 6;
            let src = mesh.indices.len() - 6;
            for i in 0..6
            {
                mesh.indices.swap(dst+i, src+i);
            }
        }
    }
}

#[derive(Clone,Copy,PartialOrd,PartialEq)]
enum FaceState
{
    NotPresent, // face is not present because it is culled, present between two voxels that are not Air
    CurrentDirection, // facing us in the current direction
    OppositeDirection, // not facing us in the current direction
}

#[derive(Clone,Copy,PartialEq,PartialOrd)]
struct SliceFace
{
    pub face_state: FaceState,
    pub voxel: Voxel,
}

impl ChunkMesher for GreedyMesher
{
    fn generate_mesh(voxels: VoxelFetcher, mesh: &mut Mesh<VoxelVertex>, trans_faces: &mut Vec<Face>)
    {
        let chunk_world_pos = voxels.get_center_chunk_pos();
        // sweep over each axis separately (X,Y,Z)

        //TODO: better documentation
        // mask coverting every voxel in the direction we are traversing, as if we took a knife and cut through the chunk perpendicular to the direction
        // we are traversing, every voxel in the cut has an entry

        // reserve the maximum number that we can use, so for the largest 2 dimensions
        let mut mask = [SliceFace{face_state:FaceState::NotPresent,voxel:Voxel::default()}; CHUNK_SIZE_X * CHUNK_SIZE_Y];

        for current_dir in 0usize..3 // 0 is X, 1 is Y, 2 is Z
        {
            let mut current_pos: IVec3 = IVec3::ZERO; // X,Y,Z
            
            let n_dir = (current_dir+1) % 3;
            let nn_dir = (current_dir+2) % 3;
            
            let mut offset : IVec3 = IVec3::ZERO;
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

                        // get the current voxel and the next one in the current direction if any
                        let current_voxel = match voxels.get_voxel(current_pos + chunk_world_pos)
                        {
                            Some(voxel) => voxel,
                            None => Voxel::new(VoxelType::Air),
                        };

                        let next_voxel = match voxels.get_voxel(current_pos + offset + chunk_world_pos)
                        {
                            Some(voxel) => voxel,
                            None => Voxel::new(VoxelType::Air),
                        };

                        // TODO: refactor jesus
                        if current_voxel.is_filled() == next_voxel.is_filled() && current_voxel.is_transparent() == next_voxel.is_transparent() // covers all no face emitted cases
                        {
                            mask[mask_index].face_state = FaceState::NotPresent;
                        }
                        else if current_voxel.is_transparent() && !next_voxel.is_transparent()
                        {
                            mask[mask_index] = SliceFace{face_state:FaceState::CurrentDirection, voxel:next_voxel}; // quad is facing us in the current direction
                        }
                        else if !current_voxel.is_transparent() && next_voxel.is_transparent()
                        {
                            mask[mask_index] = SliceFace{face_state:FaceState::OppositeDirection,voxel:current_voxel}; // quad is facing the opposite direction
                        }
                        else if current_voxel.is_transparent() && next_voxel.is_transparent() && current_voxel.voxel_type != next_voxel.voxel_type && next_voxel.is_filled()
                        {
                            mask[mask_index] = SliceFace{face_state:FaceState::CurrentDirection, voxel:next_voxel}; // quad is facing us in the current direction
                        }
                        else if current_voxel.is_transparent() && next_voxel.is_transparent() && current_voxel.voxel_type != next_voxel.voxel_type && current_voxel.is_filled()
                        {
                            mask[mask_index] = SliceFace{face_state:FaceState::OppositeDirection, voxel:current_voxel}; // quad is facing us in the current direction
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
                        let reference_face = mask[mask_index];
                        if reference_face.face_state !=  FaceState::NotPresent// if current face is visible and not transparent = it can be joined with other faces
                        {
                            // all faces that will be merged into a single large quad are identical to this reference face
                            // search along the current axis until mask[mask_index + w] is false, we are searching the quad with height 1 and the largest possible width

                            let mut width = 1;
                            let mut height = 1;

                            if !reference_face.voxel.is_transparent() // only opaque faces can be merged
                            {
                                while (i + width) < CHUNK_SIZE[nn_dir] && mask[mask_index+width] == reference_face // they must also be of the same type
                                {
                                    width += 1;
                                }
    
                                // we have the biggest width, compure the biggest height that we can have while still maintaining the current width
                                // there should be no holes in the resulting quad generated
                                'outer: while height + j < CHUNK_SIZE[n_dir]
                                {
                                    // for each height, loop over all the faces in the width making sure there are no holes
                                    for w in 0..width
                                    {
                                        if mask[mask_index + w + height * CHUNK_SIZE[nn_dir]] != reference_face // carefull
                                        {
                                            break 'outer;
                                        }
                                    }
                                    height += 1;
                                }
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

                            let chunk_world_pos = chunk_world_pos.as_vec3();

                            // append the quad
                            let upper_left = IVec3::new(current_pos[0] + dv[0],current_pos[1] + dv[1],current_pos[2] + dv[2]).as_vec3() + chunk_world_pos;
                            let lower_left = IVec3::new(current_pos[0],current_pos[1],current_pos[2]).as_vec3() + chunk_world_pos;
                            let lower_right = IVec3::new(current_pos[0] + du[0],current_pos[1] + du[1],current_pos[2] + du[2]).as_vec3() + chunk_world_pos;
                            let upper_right = IVec3::new(current_pos[0] + du[0] + dv[0],current_pos[1] + du[1] + dv[1],current_pos[2] + du[2] + dv[2]).as_vec3() + chunk_world_pos;

                            // add the face to the Transparent Face Vector
                            let face_pos = chunk_world_pos + current_pos.as_vec3() + FACE_POSITION[current_dir]; // only relevant for transparent faces which are never merged together

                            GreedyMesher::add_quad(mesh, trans_faces, face_pos, reference_face, current_dir, nn_dir, n_dir, lower_left, upper_left, upper_right, lower_right);

                            // clear the mask for each face that was used
                            for w in 0..width
                            {
                                for h in 0..height
                                {
                                    mask[mask_index + w + h * CHUNK_SIZE[nn_dir]].face_state = FaceState::NotPresent; // careful
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
    }
}