// everything related to rendering and animating the clouds and celestial bodies
// including the skybox

// ==== Assumptions ====
// East is defined by the direction vector which is the bisector of the -Z axis and +X axis
// West is defined by the direction vector which is the bisector of the +Z axis and -X axis (opposite direction of east)
// The Sun rises from the east and sets west
// The Moon rises from the east and sets west
// No particular motion or position will be set for stars

use glam::{Vec4, Vec3, Vec2, Mat4};
use core::f32::consts::PI;
use std::{time::{Instant}};
use super::{renderer::{opengl_abstractions::{shader::Shader, vertex_array::VertexLayout}, Renderer}, world::World, geometry::{mesh::{Mesh}, opengl_vertex::OpenglVertex}, voxel::VoxelVertex};

const TIME_MULTIPLIER: f32 = 1000.0;

// TODO: where to put this ?
struct SkyBoxVertex
{
    position: Vec3, // X, Y, Z
    color: Vec3, // R, G ,B
}

impl SkyBoxVertex
{
    pub fn new(position: Vec3, color: Vec3) -> Self
    {
        Self{position,color}
    }
}

impl OpenglVertex for SkyBoxVertex
{
    fn get_layout() -> VertexLayout
    {
        let mut vertex_layout = VertexLayout::new();

        vertex_layout.push_f32(3); // vertex(x,y,z)
        vertex_layout.push_f32(3); // color(r,g,b)

        vertex_layout
    }
}

struct SkyState
{
    pub start_time: f32,
    pub sun_present: bool,
    pub moon_present: bool,
    //TODO: document phi and teta
    pub pos_sun: (f32,f32), // phi and teta
    pub pos_moon: (f32,f32),
    pub sky_box_color: [Vec4;8],
}

impl SkyState
{
    pub fn lerp(&self, dest: &SkyState, s: f32) -> SkyState
    {
        // only positions and colors are lerped
        let pos_sun = (self.pos_sun.0 + (dest.pos_sun.0 - self.pos_sun.0) * s, self.pos_sun.1 + (dest.pos_sun.1 - self.pos_sun.1) * s);
        let pos_moon= (self.pos_moon.0 + (dest.pos_moon.0 - self.pos_moon.0) * s,self.pos_moon.1 + (dest.pos_moon.1 - self.pos_moon.1) * s);
        let mut sky_box_color = [Vec4::ZERO;8];

        (0..7).for_each(|i| {
            sky_box_color[i] = self.sky_box_color[i].lerp(dest.sky_box_color[i], s);
        });

        SkyState { start_time: self.start_time, sun_present: self.sun_present, moon_present: self.moon_present, pos_sun, pos_moon, sky_box_color}
    }   
}

enum DayNightPhase
{
    SunRise,
    Noon,
    Sunset,
    Night,
}

const PHASE_CONFIG : [SkyState;4] =
[
    SkyState{start_time:0.0,moon_present:false,sun_present:true,pos_moon:(0.0,0.0),pos_sun:(0.0,PI/8.0),
        sky_box_color: [Vec4::ZERO;8]},
        
    SkyState{start_time:3.0*3600.0,moon_present:false,sun_present:true,pos_moon:(0.0,0.0),pos_sun:(30.0*PI/180.0,PI/8.0),
            sky_box_color: [Vec4::ZERO;8]},

    SkyState{start_time:11.0*3600.0,moon_present:false,sun_present:true,pos_moon:(0.0,0.0),pos_sun:(150.0*PI/180.0,PI/8.0),
        sky_box_color: [Vec4::ZERO;8]},

    SkyState{start_time:12.0*3600.0,moon_present:true,sun_present:false,pos_moon:(0.0,0.0),pos_sun:(PI,0.0),
        sky_box_color: [Vec4::ZERO;8]},
];

impl TryFrom<i32> for DayNightPhase
{
    type Error = ();

    fn try_from(value: i32) -> Result<Self, Self::Error>
    {
        match value
        {
            x if x == Self::SunRise as i32 => Ok(Self::SunRise),
            x if x == Self::Noon as i32 => Ok(Self::Noon),
            x if x == Self::Sunset as i32 => Ok(Self::Sunset),
            x if x == Self::Night as i32 => Ok(Self::Night),
            _ => Err(())
        }
    }
}

impl DayNightPhase
{
    fn get_next(&self) -> DayNightPhase
    {
        let x = *self as i32;
        ((x+1)%4).try_into().expect("error converting i32 into enum")
    }
}

pub struct SkyRenderer
{
    current_phase: DayNightPhase,

    // renderer inner state
    celestial_shader: Shader,
    skybox_shader: Shader,
    sky_quad: Mesh<VoxelVertex>,
    sun_quad: Mesh<VoxelVertex>,
    sun_angle_rad: f32,
    sky_box: Mesh<SkyBoxVertex>,
    tick: f32,
    time: Instant, // time of day in seconds
}

impl SkyRenderer
{
    pub fn new() -> Self
    {
        // Initialise everything needed to render the sky + objects

        // TESTING
        let celestial_shader = Shader::new_from_vs_fs("rust-vox/shaders/celestial.vert", "rust-vox/shaders/celestial.frag").expect("Shader Error");
        // create sky plane
        let mut sky_quad = Mesh::new();
        //TODO: refactor needed, we should be able to customize the Attributes for according to each Shader
        // define it anti-clockwise, no need to rotate it in this case
        let i1 = sky_quad.add_vertex(VoxelVertex::new(Vec3::new(-1.0,0.2,1.0),0,Vec2::new(0.0,0.0)));
        let i2 = sky_quad.add_vertex(VoxelVertex::new(Vec3::new(-1.0,0.2,-1.0),0,Vec2::new(0.0,1.0)));
        let i3 = sky_quad.add_vertex(VoxelVertex::new(Vec3::new(1.0,0.2,-1.0),0,Vec2::new(1.0,1.0)));
        let i4 = sky_quad.add_vertex(VoxelVertex::new(Vec3::new(1.0,0.2,1.0),0,Vec2::new(1.0,0.0)));

        sky_quad.add_triangle(i4, i2, i1);
        sky_quad.add_triangle(i2, i4, i3);

        sky_quad.upload();

        let mut cloud_texture = 0;        
        // load texture atlas
        let img = image::open("rust-vox/textures/clouds.png").unwrap().flipv();
        let width = img.width();
        let height = img.height();
        let data = img.as_bytes();

        unsafe
        {
            gl::GenTextures(1, &mut cloud_texture);
            
            gl::ActiveTexture(gl::TEXTURE2);
            gl::BindTexture(gl::TEXTURE_2D, cloud_texture);

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as _ );
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as _ );
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as _ );
            gl::TexParameteri(gl::TEXTURE_2D ,gl::TEXTURE_MAG_FILTER, gl::NEAREST as _ );

            gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA as _ , width.try_into().unwrap() , height.try_into().unwrap() ,
                0, gl::RGBA as _ , gl::UNSIGNED_BYTE , data.as_ptr().cast() );
            gl::GenerateMipmap(gl::TEXTURE_2D);
        }

        // TESTING
        // generate the background, renderer as a unit cube around the player
        //TODO: migrate this so it uses the same shader, since even if the unit cube is renderer with
        // perspective projection, the corners are not noticable due to the small cube size
        
        let mut sky_box = Mesh::new();
        // we need 8 vertices
        // bottom 4
        // let i1 = sky_box.add_vertex(Vertex::new(Vec3::new(-1.0,-0.2,1.0),0,Vec2::new(0.0,0.0)));
        // let i2 = sky_box.add_vertex(Vertex::new(Vec3::new(-1.0,-0.2,-1.0),0,Vec2::new(0.0,0.0)));
        // let i3 = sky_box.add_vertex(Vertex::new(Vec3::new(1.0,-0.2,-1.0),0,Vec2::new(0.0,0.0)));
        // let i4 = sky_box.add_vertex(Vertex::new(Vec3::new(1.0,-0.2,1.0),0,Vec2::new(0.0,0.0)));
        // // top 4
        // let i5 = sky_box.add_vertex(Vertex::new(Vec3::new(-1.0,0.2,1.0),0,Vec2::new(1.0,0.0)));
        // let i6 = sky_box.add_vertex(Vertex::new(Vec3::new(-1.0,0.2,-1.0),0,Vec2::new(1.0,0.0)));
        // let i7 = sky_box.add_vertex(Vertex::new(Vec3::new(1.0,0.2,-1.0),0,Vec2::new(1.0,0.0)));
        // let i8 = sky_box.add_vertex(Vertex::new(Vec3::new(1.0,0.2,1.0),0,Vec2::new(1.0,0.0)));

        let i1 = sky_box.add_vertex(SkyBoxVertex::new(Vec3::new(-1.0,-0.2,1.0),Vec3::ZERO));
        let i2 = sky_box.add_vertex(SkyBoxVertex::new(Vec3::new(-1.0,-0.2,-1.0),Vec3::ZERO));
        let i3 = sky_box.add_vertex(SkyBoxVertex::new(Vec3::new(1.0,-0.2,-1.0),Vec3::ZERO));
        let i4 = sky_box.add_vertex(SkyBoxVertex::new(Vec3::new(1.0,-0.2,1.0),Vec3::ZERO));
        // top 4
        let i5 = sky_box.add_vertex(SkyBoxVertex::new(Vec3::new(-1.0,0.2,1.0),Vec3::ZERO));
        let i6 = sky_box.add_vertex(SkyBoxVertex::new(Vec3::new(-1.0,0.2,-1.0),Vec3::ZERO));
        let i7 = sky_box.add_vertex(SkyBoxVertex::new(Vec3::new(1.0,0.2,-1.0),Vec3::ZERO));
        let i8 = sky_box.add_vertex(SkyBoxVertex::new(Vec3::new(1.0,0.2,1.0),Vec3::ZERO));

        // we are sitting at the origin looking down -Z
        // define triangles
        // bottom plane
        sky_box.add_triangle(i1, i2, i4);
        sky_box.add_triangle(i4, i2, i3);
        // top plane
        sky_box.add_triangle(i8, i6, i5);
        sky_box.add_triangle(i8, i7, i6);
        // left plane
        sky_box.add_triangle(i1, i5, i2);
        sky_box.add_triangle(i2, i5, i6);
        // right plane
        sky_box.add_triangle(i3, i8, i4);
        sky_box.add_triangle(i8, i3, i7);
        // front plane
        sky_box.add_triangle(i1, i4, i5);
        sky_box.add_triangle(i5, i4, i8);
        // back plane
        sky_box.add_triangle(i2, i6, i3);
        sky_box.add_triangle(i3, i6, i7);

        sky_box.upload();

        let skybox_shader = Shader::new_from_vs_fs("rust-vox/shaders/skybox.vert",
        "rust-vox/shaders/skybox.frag" ).expect("Shader Error");

        // TESTING
        // generate the sun
        // assuming that bisector between -Z and +X is east, and bisector between +Z and -X is west
        let sun_angle_rad: f32 = 0.0; // angle in radians if looking at the sun from the north or south
        // let sun_direction = Vec3::new(1.0,0.0,-1.0);
        // TODO: continue here, adjust sun position to be at an angle with the world's geometry such that shadow's are prettiers
        
        // the sun is just a textured quad
        let mut sun_quad = Mesh::new();
        
        let i1 = sun_quad.add_vertex(VoxelVertex::new(Vec3::new(-1.0,-1.0,-5.0),0,Vec2::new(0.0,0.0)));
        let i2 = sun_quad.add_vertex(VoxelVertex::new(Vec3::new(-1.0,1.0,-5.0),0,Vec2::new(0.0,1.0)));
        let i3 = sun_quad.add_vertex(VoxelVertex::new(Vec3::new(1.0,1.0,-5.0),0,Vec2::new(1.0,1.0)));
        let i4 = sun_quad.add_vertex(VoxelVertex::new(Vec3::new(1.0,-1.0,-5.0),0,Vec2::new(1.0,0.0)));
        
        sun_quad.add_triangle(i1, i2, i4);
        sun_quad.add_triangle(i2, i3, i4);

        sun_quad.upload();

        let mut sun_texture = 0;        
        // load texture atlas
        let img = image::open("rust-vox/textures/sun.png").unwrap().flipv();
        let width = img.width();
        let height = img.height();
        let data = img.as_bytes();

        unsafe
        {
            gl::GenTextures(1, &mut sun_texture);
            
            gl::ActiveTexture(gl::TEXTURE3);
            gl::BindTexture(gl::TEXTURE_2D, sun_texture);

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as _ );
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as _ );
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as _ );
            gl::TexParameteri(gl::TEXTURE_2D ,gl::TEXTURE_MAG_FILTER, gl::NEAREST as _ );

            gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA as _ , width.try_into().unwrap() , height.try_into().unwrap() ,
                0, gl::RGBA as _ , gl::UNSIGNED_BYTE , data.as_ptr().cast() );
            gl::GenerateMipmap(gl::TEXTURE_2D);
        }

        Self{current_phase: DayNightPhase::SunRise, tick:0.0, celestial_shader,skybox_shader,sky_box,sky_quad,sun_quad,sun_angle_rad,time:Instant::now()}
    }

    fn update(&mut self) -> SkyState
    {
        //FIXME: time reset may not be correctly implemented
        // make sure to reset the timer accordingly
        if self.time.elapsed().as_secs_f32() >= 24.0*3600.0
        {
            self.time = Instant::now(); // restart timer
        }

        let mut current_phase = &PHASE_CONFIG[self.current_phase as usize];
        let next_phase = &PHASE_CONFIG[self.current_phase.get_next() as usize];

        let mut progress = (self.time.elapsed().as_secs_f32() * TIME_MULTIPLIER - current_phase.start_time) / (next_phase.start_time - current_phase.start_time);
        
        if progress >= 1.0 // current phase is done, proceed to next phase
        {
            println!("next phase!");
            self.current_phase = self.current_phase.get_next();
            current_phase = &PHASE_CONFIG[self.current_phase as usize];
            progress = 0.0;
        }

        // interpolate positions and colors between the two phases
        
        current_phase.lerp(next_phase, progress)
    }

    pub fn render(&mut self, world: &World)
    {
        let sky_state = self.update();

        // PASS 4: draw skybox
        self.skybox_shader.bind();
        Renderer::draw_mesh(&self.sky_box);
        Shader::unbind();

        // PASS 3: draw celestial bodies
        unsafe
        {
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA); 
        }

        // remove the translation component from the camera's view matrix, since the background
        // must appear the same from any camera position
        // this done by taking the upper 3x3 matrix from original 4x4 view matrix
        self.celestial_shader.bind();
        self.tick += 0.0001;
        if self.tick > 1.0 {self.tick = 0.0;}
        self.celestial_shader.set_uniform_1f("sub", self.tick).expect("error setting sub float uniform");
        self.celestial_shader.set_uniform1i("text", 2).expect("error setting the sky texture");
        self.celestial_shader.set_uniform_matrix4fv("model", &Mat4::IDENTITY).expect("error setting the view uniform");
        Renderer::draw_mesh(&self.sky_quad);
        self.update_sun();
        self.celestial_shader.set_uniform_1f("sub", 0.0).expect("error setting sub float uniform");
        self.celestial_shader.set_uniform1i("text", 3).expect("error setting the sun texture");
        // println!("sun coords: {:?}", sky_state.pos_sun);
        let sun_quad_trans = Mat4::from_rotation_x(sky_state.pos_sun.0) * Mat4::from_rotation_y(sky_state.pos_sun.1);
        self.celestial_shader.set_uniform_matrix4fv("model", &sun_quad_trans).expect("error setting the model transformation for the sun_quad");
        Renderer::draw_mesh(&self.sun_quad);
        unsafe {gl::Disable(gl::BLEND);}

        // Buffer Respecification (Orphaning)
        // 
        // gl::BindBuffer()
        // gl::BufferData(gl::ARRAY_BUFFER, data_size, NULL, gl::STREAM_DRAW)
        // gl::BufferData(gl::ARRAY_BUFFER, data_size, mydata_ptr, gl::STREAM_DRAW) use same spec arguments
    }

    //TODO: this does not belong here obviously
    fn update_sun(&mut self)
    {
        self.sun_angle_rad += 0.001; // TODO: should be set according to the frame rate
        if self.sun_angle_rad >= 2.0*PI
        {
            self.sun_angle_rad -= 2.0*PI;
        }
        
        // adjust the sun's direction angle
        
        let y_coord = self.sun_angle_rad.sin();
        let along_bisector = self.sun_angle_rad.cos();
        // project the bisector onto X and Z
        let x_coord = along_bisector * (PI/8.0).cos();
        let z_coord = along_bisector * (PI/8.0).sin();

        // self.sun_direction = Vec3::new(x_coord,y_coord,z_coord);
        // println!("[DEBUG] sun direction: {}", self.sun_direction);
    }
}

