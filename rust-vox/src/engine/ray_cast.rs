// is used when placing/removing blocks to find the voxel the player is pointing at

// From "A Fast Voxel Traversal Algorithm for Ray Tracing"
// by John Amanatides and Andrew Woo, 1987

use glam::{Vec3};
use crate::engine::geometry::voxel::Voxel;
use super::chunk_manager::ChunkManager;

// uses get_closest_voxel
pub fn cast_ray(position: Vec3, direction: Vec3, chunk_manager: &ChunkManager) -> Option<(Vec3,Vec3)>
{
    let mut found = false;
    let mut used_position = Vec3::ZERO;
    let mut used_face = Vec3::ZERO;

    let mut used_voxel = Voxel::default();

    println!("***********");

    get_closest_voxel(position, direction, 20.0, // 10 voxels only 
        |pos,fa|
        {
            println!("position: {}, face:{}", used_position, used_face);
            used_position = pos;
            used_face = fa;

            if let Some(voxel) = chunk_manager.get_voxel(pos)
            {
                if voxel.is_filled()
                {
                    used_voxel = voxel;
                    println!("Found filled voxel at pos: {}, entered from face: {}", pos, fa);
                    println!("voxel: {:?}", voxel.voxel_type);

                    found = true;
                    true
                }
                else
                {
                    false
                }
            }
            else
            {
                false
            }
        }
    );

    if found {Some((used_position,used_face))} else {None}
}

pub fn get_closest_voxel<T> (origin: Vec3, direction: Vec3, max_radius: f32, mut callback: T)
    where T: FnMut(Vec3, Vec3) -> bool
{
    if direction == Vec3::ZERO
        {return;}

    // direction to increment X,Y and Z when stepping
    let step_x = direction.x.signum();
    let step_y = direction.y.signum();
    let step_z = direction.z.signum();

    let step = Vec3::new(step_x,step_y,step_z);

    // the initial steps needed to get to the next voxel in each respective direction
    let tmax_x = init_step(origin.x,direction.x);
    let tmax_y = init_step(origin.y, direction.y);
    let tmax_z = init_step(origin.z, direction.z);

    let mut tmax = Vec3::new(tmax_x,tmax_y,tmax_z);

    let tdelta_x = step_x/direction.x;
    let tdelta_y = step_y/direction.y;
    let tdelta_z = step_z/direction.z;

    let delta = Vec3::new(tdelta_x,tdelta_y,tdelta_z);

    // coordinates of the voxel the ray is originating in (in voxel coordinates)
    let mut current_voxel = origin.floor();

    let mut face: Vec3;

    // the limit is currently expressed in units of unit voxel
    // we need the limit to be expressed as the max value that t can have
    // simply divide the given limit by length of the vector formed by t = 1
    let max_radius = max_radius / direction.length();
    loop
    {
        face = Vec3::ZERO;
    
        // find the direction along which required the smallest t to get to the next voxel, this is next direction
        // 0 is X, 1 is Y, 2 is Z
        //TODO: refactor, finding index of the smallest element in the array
        let dir = if tmax[0] < tmax[1] {if tmax[0] < tmax[2] {0} else {2}} else if tmax[1] < tmax[2] {1} else {2};

        if tmax[dir] > max_radius
        {
            println!("exceeded radius");
            break; // exceeded limits, exit
        }
        
        // if next the next step if  > radius quick altogether
        current_voxel[dir] += step[dir];

        // now, the distance along this direction to the next voxel boundary is increment by delta
        tmax[dir] += delta[dir];

        // record the face
        face[dir] = -step[dir]; // this is the face from where we entered the next voxel
        
        if callback(current_voxel, face)
        {
            break; // we can stop
        }
    }
}

fn init_step(origin: f32, direction: f32) -> f32
{
    if direction == 0.0
    {
        return f32::INFINITY;
    }

    let opposite = origin.is_sign_negative() != direction.is_sign_negative();
    let origin = origin.fract().abs();
    
    let next_distance = if opposite {origin} else {1.0 - origin};
    next_distance / direction.abs()
}