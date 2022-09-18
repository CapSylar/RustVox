// contains the implementation of Cascaded Shadow Maps

use std::{ffi::c_void, mem::size_of};
use glam::{Mat4, Vec3, Vec4, Vec4Swizzles};
use crate::engine::eye::{Eye};

pub struct Csm
{
    cascades: Vec<f32>,
    cascade_bounds: Vec<(Vec3,f32)>,
    prev_view_trans: Vec<Mat4>,

    // depth texture paramters
    width: i32,
    height:i32,

    // light parameters
    light_direction: Vec3,

    // opengl objects
    matrices_ubo: u32,
    depth_texture_array: u32,

    // local cache
    light_space_matrices: Vec<Mat4>,
}

impl Csm
{
    // TODO: create a new() where the partitions are passed in

    pub fn new( width: i32, height: i32, eye: &Eye, light_direction: Vec3) -> Self
    {
        let near_plane = eye.near_plane;
        let far_plane = eye.far_plane;

        // Create a uniform buffer containing the light_space tranform matrices for cascaded shadow mapping
        let mut matrices_ubo = 0;
        unsafe
        {
            gl::GenBuffers(1, &mut matrices_ubo);
            gl::BindBuffer(gl::UNIFORM_BUFFER, matrices_ubo);
            gl::BufferData(gl::UNIFORM_BUFFER, (size_of::<Mat4>()*8).try_into().unwrap(), std::ptr::null::<c_void>(), gl::STATIC_DRAW);
            gl::BindBufferBase(gl::UNIFORM_BUFFER,0,matrices_ubo);
            gl::BindBuffer(gl::UNIFORM_BUFFER, 0);
        }

        // define cascades
        let cascades = vec![near_plane,far_plane/50.0,far_plane/25.0,far_plane/10.0,far_plane/2.0,far_plane];
        let prev_view_trans = vec![Mat4::IDENTITY;cascades.len()-1];
        let light_space_matrices = vec![Mat4::IDENTITY;cascades.len()-1];
        let mut cascade_bounds = vec![(Vec3::ZERO,0.0);cascades.len()-1];

        let mut i = 0;
        while i < cascades.len()-1
        {
            cascade_bounds[i] = Self::precalculate_cascade_center(cascades[i],cascades[i+1], eye.fov_y, eye.aspect_ratio);
            i += 1;
        }
    
        // allocate a 3d texture for the depth textures
        let mut depth_texture_array = 0;

        unsafe
        {
            gl::GenTextures(1, &mut depth_texture_array);
            gl::BindTexture(gl::TEXTURE_2D_ARRAY, depth_texture_array);
            gl::TexImage3D(gl::TEXTURE_2D_ARRAY, 0, gl::DEPTH_COMPONENT32F as _ , width, height,
                (cascades.len()-1).try_into().unwrap(), 0 , gl::DEPTH_COMPONENT, gl::FLOAT , std::ptr::null::<c_void>() );

            gl::TexParameteri(gl::TEXTURE_2D_ARRAY, gl::TEXTURE_MIN_FILTER,gl::NEAREST as _ );
            gl::TexParameteri(gl::TEXTURE_2D_ARRAY, gl::TEXTURE_MAG_FILTER,gl::NEAREST as _ );
            gl::TexParameteri(gl::TEXTURE_2D_ARRAY, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_BORDER as _ );
            gl::TexParameteri(gl::TEXTURE_2D_ARRAY, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_BORDER as _ );
            let border_color: [f32;4] = [1.0,1.0,1.0,1.0];
            gl::TexParameterfv(gl::TEXTURE_2D_ARRAY, gl::TEXTURE_BORDER_COLOR, border_color.as_ptr() );
            gl::BindTexture(gl::TEXTURE_2D_ARRAY, 0); // unbind
        }

        Self{ cascades, prev_view_trans, depth_texture_array, matrices_ubo, light_space_matrices, cascade_bounds, width, height, light_direction }
    }

    pub fn update(&mut self , eye: &Eye)
    {
        // calculate the updated light-space matrices for each cascasde
        self.get_cascaded_lightspace_matrices(eye);
        unsafe
        {
            // upload the matrices into the Uniform Buffer
            gl::BindBuffer(gl::UNIFORM_BUFFER, self.matrices_ubo);
            gl::BufferSubData(gl::UNIFORM_BUFFER,0,(self.light_space_matrices.len() * size_of::<Mat4>()).try_into().unwrap(),self.light_space_matrices.as_ptr() as _);
            gl::BindBuffer(gl::UNIFORM_BUFFER,0); // unbind
        }
    }

    /// Calculate the center and radius of a bounding sphere for the passed in cascade
    fn precalculate_cascade_center(n: f32, f: f32, fov_y: f32, aspect_ratio: f32) -> (Vec3,f32)
    {
        // calculate a bounding sphere for the frustum
        let k = f32::sqrt(1.0 + aspect_ratio * aspect_ratio) * (fov_y/2.0).tan();
        
        let center: Vec3;
        let radius: f32 ;

        if k*k >= (f-n)/(f+n) // if near plane is too close to the far plane, the center should be on the far plane
        {
            center = Vec3::new(0.0,0.0,-f);
            radius = k*f;
        }
        else
        {
            center = Vec3::new(0.0,0.0,(-0.5)*(f+n)*(1.0+k*k));
            radius = 0.5 * f32::sqrt((f-n)*(f-n) + 2.0*k*k*(f*f+n*n) + (f+n)*(f+n)*k*k*k*k);
        }

        (center,radius)
    }

    /// Generated matrices go into the passed in "matrices_out" slice
    /// Assumes cascades.len > 2
    fn get_cascaded_lightspace_matrices(&mut self, eye: &Eye)
    {
        // first cascade starts from the near plane
        let mut i : usize = 0;

        while i < self.cascades.len()-1
        {
            self.light_space_matrices[i] = Self::get_lightspace_transformation(&self.cascade_bounds[i],eye, self.width, self.light_direction, &mut self.prev_view_trans[i]);
            i += 1;
        }
    }

    /// Get the transformation matrix that transforms the world to "light space"
    /// The directional light looks at the center of the frustum
    fn get_lightspace_transformation( bound_sphere: &(Vec3,f32) , eye: &Eye, texture_texel_size: i32, light_direction: Vec3, prev_view_trans_out: &mut Mat4) -> Mat4
    {
        let center = bound_sphere.0;
        let radius = bound_sphere.1;

        // transform the center from camera's view space -> world space
        let mut center_world = eye.get_look_at().inverse() * Vec4::new(center.x, center.y, center.z,1.0);
        center_world /= center_world.w; // transform to cartesian coordinates

        // snap the current center to texel offsets from the last center
        // calculate the direction old -> new
        
        // get shadow map resolution
        let resol : f32 = (2.0 * radius) / texture_texel_size as f32 ;

        let mut new_center_old_view = *prev_view_trans_out * center_world;
        // snap the coordinates to texel offsets
        new_center_old_view.x = f32::floor(new_center_old_view.x / resol) * resol;
        new_center_old_view.y = f32::floor(new_center_old_view.y / resol) * resol;
        
        // transform back to world space and get the correct world position of the new center
        let mut corrected_center_world = prev_view_trans_out.inverse() * new_center_old_view;

        // get the new look at transform
        corrected_center_world /= corrected_center_world.w;

        let center = corrected_center_world.xyz();

        // println!("center for near_plane: {}, far_plane: {}, center : {}, radius: {}", n , f , center , radius);

        let light_look_at = Mat4::look_at_rh(center + light_direction, center,Vec3::new(0.0,1.0,0.0));

        *prev_view_trans_out = light_look_at; // set current as previous

        // get the 3D AABB (Axis Aligned Bounding Box) for the view frustum, the bounding box should enclose the sphere

        let radius: f32 = (radius / resol).floor() * resol;
        
        // now we construct the light's orthographic projection matrix
        let light_ortho_proj = Mat4::orthographic_rh_gl(-radius, radius,
        -radius, radius,
        -radius*1.05, radius);  // pull the near plane back a bit TODO: document
        
        light_ortho_proj * light_look_at
    }

    /// Gets the world-space coordinates of the frustum
    /// the output is filled in the passed-in corners array
    fn _get_worldspace_frustum_corners(proj_view_matrix: &Mat4, corners: &mut [Vec4;8])
    {
        let inverse = proj_view_matrix.inverse();
        // transform the NDC frustum corner [-1,1] in each axis to their corresponding world space coordinates
        let mut index = 0;
        for x in 0..2
        {
            for y in 0..2
            {
                for z in 0..2
                {
                    let pt = inverse * Vec4::new((2*x-1) as f32,(2*y-1) as f32,(2*z-1) as f32,1.0);
                    corners[index] = pt/pt.w;
                    index += 1;
                }
            }
        }
    }

    // getter, setters

    pub fn get_depth_texture_id (&self) -> u32 { self.depth_texture_array }
    pub fn get_cascade_levels(&self) -> &Vec<f32> { &self.cascades }

    pub fn get_height(&self) -> i32 { self.height }
    pub fn get_width(&self) -> i32 { self.width }

}