use std::{ffi::{c_void, CStr, CString}, f32::consts::PI};
use glam::{Mat4, Vec4, Vec3, };
use sdl2::{VideoSubsystem};

pub mod vertex_buffer;
pub mod index_buffer;
pub mod vertex_array;
pub mod shader;

use crate::engine:: {chunk::Chunk};
use self::{vertex_array::{VertexArray, VertexBufferLayout}, shader::Shader};

use super::{world::World, mesh::Mesh};

pub struct Renderer
{
    shader : Shader ,
    texture1: u32,
    chunk: Chunk,
    // mode: u32, // filled or lines 
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
            // TODO: what is extern "system" fn ? 
            gl::DebugMessageCallback( Some(error_callback) , 0 as *const c_void);
        }

        unsafe
        {
            let mut texture1 = 0;
            let mut chunk = Chunk::new(Vec3::new(0.0, 0.0, 0.0));

            // testing a single voxel
            chunk.mesh.upload();
        
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

            gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGB as _ , width.try_into().unwrap() , height.try_into().unwrap() ,
                0, gl::RGB as _ , gl::UNSIGNED_BYTE , data.as_ptr().cast() );
            gl::GenerateMipmap(gl::TEXTURE_2D);

            // ------------------------------------------ END 

            gl::BindVertexArray(0); // unbind VAO
            gl::BindBuffer(gl::ARRAY_BUFFER , 0); // unbind currently bound buffer

            // load the program

            let shader = Shader::new("rust-vox/shaders/default_vertex.glsl".to_string(),
             "rust-vox/shaders/default_fragment.glsl".to_string() ).expect("Shader Error");
            
            gl::Enable(gl::DEPTH_TEST);
            // gl::Enable(gl::CULL_FACE);
            gl::FrontFace(gl::CW);

            Renderer { shader , texture1 , chunk }
        }


    }

    pub fn draw_world(&mut self, world: &World)
    {
        //TODO: remove these from here
        let texture = CString::new("test_texture1").unwrap();
        let transform = CString::new("transform").unwrap();

        unsafe
        {
            gl::ClearColor(0.2,0.2,0.2,1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            
            // set program as current 
            self.shader.bind();
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, self.texture1);

            self.shader.set_uniform1i(&texture, 0).expect("error setting the texture uniform");

            let projection = Mat4::perspective_rh_gl(PI/4.0, 800.0/600.0, 0.1, 100.0);
            // camera matrix 
            let trans =  projection * world.camera.get_look_at();
            self.shader.set_uniform_matrix4fv(&transform, &trans).expect("error setting the transform uniform");
            
            // draw each chunk's mesh
            Renderer::draw_mesh(&self.chunk.mesh);
            
            Shader::unbind();
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