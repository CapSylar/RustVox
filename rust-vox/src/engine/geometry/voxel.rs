#[derive(Clone,Copy,PartialOrd, PartialEq, Eq,Debug)]
pub enum VoxelType
{
    Dirt,
    Sand,
    Air,
}

#[derive(Clone,Copy)]
pub struct Voxel
{
    pub voxel_type : VoxelType,
}

impl Voxel
{
    pub fn new(voxel_type: VoxelType) -> Voxel
    {
        Voxel{voxel_type}
    }

    pub fn default() -> Voxel
    {
        Voxel { voxel_type: VoxelType::Dirt}
    }

    pub fn is_filled(&self) -> bool
    {
        !(self.voxel_type == VoxelType::Air)
    }

    pub fn set_type(&mut self, voxel_type : VoxelType ) {self.voxel_type = voxel_type }
}