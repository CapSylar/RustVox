
#[derive(Clone,Copy,PartialOrd, PartialEq, Eq,Debug)]
pub enum VoxelType
{
    Dirt,
    Sand,
    Air,
    Water
}

// manual definition of block attributes for now
const VOXEL_ATTRIBUTES : [VoxelAttribs;4] =[VoxelAttribs::new(true, false), // dirt
                                            VoxelAttribs::new(true, false), // sand
                                            VoxelAttribs::new(false, true), // air
                                            VoxelAttribs::new(true, true)]; // water

struct VoxelAttribs
{
    is_filled: bool,
    is_transparent: bool,
}

impl VoxelAttribs
{
    const fn new(is_filled: bool, is_transparent: bool) -> Self
    {
        Self{is_filled, is_transparent}
    }
}

#[derive(Clone,Copy)]
pub struct Voxel
{
    pub voxel_type : VoxelType,
}

impl Default for Voxel
{
    fn default() -> Self
    {
        Self { voxel_type: VoxelType::Dirt }
    }
}

impl Voxel
{
    pub fn new(voxel_type: VoxelType) -> Voxel
    {
        Voxel{voxel_type}
    }

    pub fn is_filled(&self) -> bool
    {
        VOXEL_ATTRIBUTES[self.voxel_type as usize].is_filled
    }

    pub fn is_transparent(&self) -> bool
    {
        VOXEL_ATTRIBUTES[self.voxel_type as usize].is_transparent
    }

    pub fn set_type(&mut self, voxel_type : VoxelType ) {self.voxel_type = voxel_type }
}