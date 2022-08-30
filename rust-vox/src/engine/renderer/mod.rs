use std::{ffi::{c_void, CStr}, f32::consts::PI, mem::size_of};
use glam::{Mat4, Vec3, Vec4, Vec4Swizzles, Vec2};
use sdl2::{VideoSubsystem};

pub mod vertex_buffer;
pub mod index_buffer;
pub mod vertex_array;
pub mod shader;

use self::{vertex_array::{VertexArray}, shader::Shader};
use super::{world::{World}, mesh::Mesh, camera::{Camera}};

pub struct Renderer
{
    shadow_fb: u32,
    matrices_ubo: u32,
    default_shader : Shader,
    shadow_shader : Shader,
    texture1: u32,
    depth_texture: u32,
    cascade_levels: Vec<f32>,
}

impl Renderer
{
    pub fn new(video_subsystem: &VideoSubsystem) -> Renderer
    {
        // Setup
        // load up every opengl function, is this good ?
        gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as _);

        unsafe
        {
            gl::Enable(gl::DEBUG_OUTPUT);
            gl::DebugMessageCallback( Some(error_callback) , 0 as *const c_void);
        }

        unsafe
        {
            let mut texture1 = 0;        
            // load texture atlas
            let img = image::open("rust-vox/textures/atlas.png").unwrap().flipv();
            let width = img.width();
            let height = img.height();
            let data = img.as_bytes();

            gl::GenTextures(1, &mut texture1);
            
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, texture1);

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as _ );
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as _ );
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST_MIPMAP_NEAREST as _ );
            gl::TexParameteri( gl::TEXTURE_2D , gl::TEXTURE_MAG_FILTER , gl::NEAREST as _ );

            gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA as _ , width.try_into().unwrap() , height.try_into().unwrap() ,
                0, gl::RGBA as _ , gl::UNSIGNED_BYTE , data.as_ptr().cast() );
            gl::GenerateMipmap(gl::TEXTURE_2D);

            // ------------------------------------------ END 

            gl::BindVertexArray(0); // unbind VAO
            gl::BindBuffer(gl::ARRAY_BUFFER , 0); // unbind currently bound buffer

            // load the program

            let default_shader = Shader::new_from_vs_fs("rust-vox/shaders/default.vert",
             "rust-vox/shaders/default.frag" ).expect("Shader Error");
            
            gl::Enable(gl::DEPTH_TEST);
            gl::Enable(gl::CULL_FACE);
            gl::FrontFace(gl::CW);

            let shadow_shader =  Shader::new_from_vs_gs_fs("rust-vox/shaders/shadow.vert",
            "rust-vox/shaders/shadow.geom", "rust-vox/shaders/shadow.frag" ).expect("Shader Error");

            // generate a framebuffer for the shadow map

            let mut shadow_fb = 0;
            gl::GenFramebuffers(1, &mut shadow_fb);

            // TODO: remove parameters from here
            let shadow_height: i32 = 2048;
            let shadow_width: i32 = 2048;
            // testing cascades

            let far_plane: f32 = 500.0;
            let near_plane: f32 = 0.1;
            // cascades include the original near and far planes
            // let cascades = vec![near_plane,far_plane/50.0,far_plane/20.0,far_plane/5.0,far_plane];
            let cascades = vec![near_plane,far_plane];

            // Create a uniform buffer containing the light_space tranform matrices for cascaded shadow mapping
            let mut matrices_ubo = 0;
            gl::GenBuffers(1, &mut matrices_ubo);
            gl::BindBuffer(gl::UNIFORM_BUFFER, matrices_ubo);
            gl::BufferData(gl::UNIFORM_BUFFER, (size_of::<Mat4>()*8).try_into().unwrap(), 0 as * const c_void, gl::STATIC_DRAW);
            gl::BindBufferBase(gl::UNIFORM_BUFFER,0,matrices_ubo);
            gl::BindBuffer(gl::UNIFORM_BUFFER, 0);

            let mut depth_texture_array = 0;
            gl::GenTextures(1, &mut depth_texture_array);
            gl::BindTexture(gl::TEXTURE_2D_ARRAY, depth_texture_array);

            gl::TexImage3D(gl::TEXTURE_2D_ARRAY, 0, gl::DEPTH_COMPONENT32F as _ , shadow_width, shadow_height,
               (cascades.len()-1).try_into().unwrap(), 0 , gl::DEPTH_COMPONENT, gl::FLOAT , 0 as * const c_void );

            gl::TexParameteri(gl::TEXTURE_2D_ARRAY, gl::TEXTURE_MIN_FILTER,gl::NEAREST as _ );
            gl::TexParameteri(gl::TEXTURE_2D_ARRAY, gl::TEXTURE_MAG_FILTER,gl::NEAREST as _ );
            gl::TexParameteri(gl::TEXTURE_2D_ARRAY, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_BORDER as _ );
            gl::TexParameteri(gl::TEXTURE_2D_ARRAY, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_BORDER as _ );
            let border_color: [f32;4] = [1.0,1.0,1.0,1.0];
            gl::TexParameterfv(gl::TEXTURE_2D_ARRAY, gl::TEXTURE_BORDER_COLOR, border_color.as_ptr() );

            gl::BindFramebuffer(gl::FRAMEBUFFER, shadow_fb);
            gl::FramebufferTexture(gl::FRAMEBUFFER, gl::DEPTH_ATTACHMENT, depth_texture_array, 0);
            // need to explicitely mention that we will render no color on this framebuffer
            gl::DrawBuffer(gl::NONE);
            gl::ReadBuffer(gl::NONE);
            // unbind
            gl::BindFramebuffer( gl::FRAMEBUFFER, 0);

            Renderer { default_shader , shadow_shader , texture1 , shadow_fb , depth_texture: depth_texture_array , cascade_levels:cascades, matrices_ubo }
        }
    }

    pub fn draw_world(&mut self, world: &World)
    {   
        let projection = Mat4::perspective_rh_gl(PI/4.0, 800.0/600.0, 0.1, 500.0);
        let view = world.camera.get_look_at();


        unsafe
        {
            // PASS 1: render to the shadow map
            self.render_shadow(world);
            
            // PASS 2: render the scene normally
            self.default_shader.bind();

            gl::BindFramebuffer(gl::FRAMEBUFFER, 0); // bind default framebuffer
            gl::Viewport(0, 0, 1700 ,900);
            gl::ClearColor(0.25,0.5,0.88,1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            gl::Enable(gl::CULL_FACE);
            gl::CullFace(gl::BACK);
            
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, self.texture1);
            gl::ActiveTexture(gl::TEXTURE1);
            gl::BindTexture(gl::TEXTURE_2D_ARRAY, self.depth_texture);

            // FIXME: setting the constant uniforms at every draw ?
            // self.default_shader.set_uniform1i("texture_atlas", 0).expect("error setting the texture uniform");
            self.default_shader.set_uniform1i("shadow_map", 1).expect("error setting the shadow map texture uniform");
            self.default_shader.set_uniform1i("cascade_count", self.cascade_levels.len() as i32 ).expect("error setting the cascade count");
            self.default_shader.set_uniform_1fv("cascades", &self.cascade_levels).expect("error setting the cascades");

            // camera matrix 
            // let trans =  projection * view;
            self.default_shader.set_uniform_matrix4fv("view", &view).expect("error setting the view uniform");
            self.default_shader.set_uniform_matrix4fv("perspective", &projection).expect("error setting the perspective uniform");

            Self::draw_geometry(world, &mut self.default_shader);
            Shader::unbind();
        }
    }

    fn draw_geometry(world: &World, shader: &mut Shader)
    {
        // draw each chunk's mesh
        for chunk in world.chunk_manager.get_chunks_to_render().iter()
        {
            let chunk = chunk.borrow_mut();
            if let Some(ref offset) = chunk.animation
            {
                shader.set_uniform3fv("animation_offset", &offset.current ).expect("error setting animation offset!");
            }
            else
            {
                shader.set_uniform3fv("animation_offset", &Vec3::ZERO).expect("error setting animation offset!");
            }

            Renderer::draw_mesh(chunk.mesh.as_ref().expect("mesh was not initialized!"));
        }
    }

    /// Depth Only Render Pass
    /// Implements Cascaded Shadow Maps (CSM)
    fn render_shadow(&mut self, world: &World)
    {
        let mut matrices = Vec::new();
        Self::get_cascaded_lightspace_matrices(&self.cascade_levels, world, &mut matrices);
    
        unsafe
        {
            // upload the matrices into the Uniform Buffer
            gl::BindBuffer(gl::UNIFORM_BUFFER, self.matrices_ubo);
            gl::BufferSubData(gl::UNIFORM_BUFFER,0,(matrices.len() * size_of::<Mat4>()).try_into().unwrap(),matrices.as_ptr() as _);
            gl::BindBuffer(gl::UNIFORM_BUFFER,0); // unbind
        }

        self.shadow_shader.bind();

        unsafe
        {   
            gl::Viewport(0, 0, 2048, 2048);
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.shadow_fb);
            gl::Clear(gl::DEPTH_BUFFER_BIT);
            gl::Disable(gl::CULL_FACE);
        }

        Self::draw_geometry(world,&mut self.shadow_shader);

        unsafe
        {
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }
    }

    /// Gets the world-space coordinates of the frustum
    /// the output is filled in the passed-in corners array
    fn get_worldspace_frustum_corners(proj_view_matrix: &Mat4, corners: &mut [Vec4;8])
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

    /// Get the transformation matrix that transforms the world to "light space"
    /// The directional light looks at the center of the frustum
    fn get_lightspace_transformation(near_plane: f32, far_plane: f32, camera: &Camera) -> Mat4
    {
        let projection = Mat4::perspective_rh_gl(PI/4.0, 800.0/600.0, near_plane, far_plane);
        let proj_view = projection * camera.get_look_at();

        let mut corners: [Vec4;8] = [Vec4::ZERO;8];
        Self::get_worldspace_frustum_corners(&proj_view, &mut corners);
        
        let mut center: Vec3 = Vec3::ZERO;
        for point in corners
        {
            center += point.xyz();
        }
        // get the average
        center /= corners.len() as f32;

        println!("center for near_plane: {}, far_plane: {} center : {}", near_plane , far_plane , center); 

        let sun_direction = Vec3::new(0.5,0.2,0.0);
        let light_look_at = Mat4::look_at_rh(center + sun_direction,center,Vec3::new(0.0,1.0,0.0));

        // get the 3D AABB (Axis Aligned Bounding Box) for the view frustum

        let mut min_x = f32::MAX;
        let mut max_x = f32::MIN;
        let mut min_y = f32::MAX;
        let mut max_y = f32::MIN;
        let mut min_z = f32::MAX;
        let mut max_z = f32::MIN;

        for mut point in corners
        {
            // first transform the points from world space to the light's view space
            point = light_look_at * point; // the point in the light's view space
            min_x = f32::min(min_x,point.x);
            max_x = f32::max(max_x,point.x);
            min_y = f32::min(min_y,point.y);
            max_y = f32::max(max_y,point.y);
            min_z = f32::min(min_z,point.z);
            max_z = f32::max(max_z,point.z);
        }

        println!("min_x: {}, max_x: {}, size: {}", min_x , max_x, max_x-min_x);
        println!("min_y: {}, max_y: {}, size: {}", min_y , max_y, max_y-min_y);
        // let units_per_texel = 2.0 * Vec2::new( max_x - min_x, max_y - min_y ) / Vec2::new( 2048.0 , 2048.0 );

        // min_x = f32::floor( min_x / units_per_texel.x ) * units_per_texel.x;
        // max_x = f32::floor( max_x / units_per_texel.x ) * units_per_texel.x;

        // min_y = f32::floor( min_y / units_per_texel.y ) * units_per_texel.y;
        // max_y = f32::floor( max_y / units_per_texel.y ) * units_per_texel.y;

        // println!("units per pixel: x:{} , y:{}", units_per_texel.x, units_per_texel.y );
        
        // now we construct the light's orthographic projection matrix
        let light_ortho_proj = Mat4::orthographic_rh_gl(min_x, max_x, min_y, max_y, min_z, max_z);
        
        light_ortho_proj * light_look_at
    }

    /// Generated matrices go into the passed in "matrices" Vec
    /// Assumes cascades.len > 2
    fn get_cascaded_lightspace_matrices( cascades: &Vec<f32>, world: &World, matrices: &mut Vec<Mat4> )
    {
        // first cascade starts from the near plane
        // matrices.push(Self::get_lightspace_transformation(cascades[0], cascades[1], &world.camera));
        let mut i : usize = 0; // we already added one frustum

        while i < cascades.len()-1
        {
            matrices.push(Self::get_lightspace_transformation(cascades[i], cascades[i+1], &world.camera));
            i += 1;
        }
    }

    pub fn draw_mesh(mesh: &Mesh)
    {
        unsafe
        {
            mesh.vao.as_ref().unwrap().bind();
            gl::DrawElements(gl::TRIANGLES, mesh.indices.len() as _  , gl::UNSIGNED_INT, 0 as _ );
            VertexArray::unbind();
        }
    }

    //FIXME: problematic interface
    pub fn set_mode(&mut self, mode: u32)
    {
        unsafe
        {
            gl::PolygonMode(gl::FRONT_AND_BACK, mode );
        }
    }


}

// error callback function for opengl
extern "system" fn error_callback ( _source : u32 , error_type : u32 , _id : u32 , _severity : u32 , _len : i32 , message: *const i8 , _user_param : *mut c_void )
{
    unsafe
    {
        if error_type == gl::DEBUG_TYPE_ERROR
        {
            let x = CStr::from_ptr(message).to_string_lossy().to_string();
            println!("ERROR CALLBACK: {}" , x);
        }
    }
}