use glam::{IVec2, IVec3};

use crate::engine::{geometry::{meshing::voxel_fetcher::VoxelSetter, voxel::{Voxel, VoxelType}}, chunk::CHUNK_SIZE_Y, management::chunk_manager::ChunkManager};

pub fn decorate_chunk(chunk_pos: IVec2, mut voxel_setter: VoxelSetter)
{
    // put a tree at pos 0,0,0 in the chunk
    // for now make the trunk out of glass
    let world_pos = ChunkManager::chunk_to_world_coord(chunk_pos);

    let mut surface_y: i32 = 0;

    for y in 0..CHUNK_SIZE_Y
    {
        let vox = voxel_setter.get_voxel(world_pos + IVec3::new(0,y as i32,0)).unwrap();

        if vox == Voxel::new(VoxelType::Air)
        {
            surface_y = y as i32;
            break;
        }
    }

    let tree_length = 7;
    let tree_x = 5;
    let tree_z = 5;
    
    for y in 0..tree_length
    {
        voxel_setter.set_voxel(world_pos + IVec3::new(0,surface_y + y,0), Voxel::new(VoxelType::Sand));
    }

    for y in surface_y + tree_length-3..surface_y + tree_length
    {
        for x in -tree_x/2..tree_x/2 +1
        {
            for z in -tree_z/2..tree_z/2 + 1
            {
                if x == 0 && z == 0
                {
                    continue; // do not overwrote the tree 
                }

                let pos = world_pos + IVec3::new(x,y,z);
                voxel_setter.set_voxel(pos, Voxel::new(VoxelType::Leaves));
            }
        }
    }
}