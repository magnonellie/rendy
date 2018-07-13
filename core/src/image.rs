
use ash;

pub type Type = ash::vk::ImageType;
pub type Extent3D = ash::vk::Extent3D;
pub type Layout = ash::vk::ImageLayout;
pub type Usage = ash::vk::ImageUsageFlags;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum Kind {
    D1(u32),
    D2 {
        width: u32,
        height: u32,
    },
    D3 {
        width: u32,
        height: u32,
        depth: u32,
    }
}

impl Kind {
    /// Get type of the image.
    pub fn image_type(self) -> Type {
        match self {
            Kind::D1(_) => ash::vk::ImageType::Type1d,
            Kind::D2 {..} => ash::vk::ImageType::Type2d,
            Kind::D3 {..} => ash::vk::ImageType::Type2d,
        }
    }

    /// Get extent of the image.
    pub fn extent(self) -> Extent3D {
        match self {
            Kind::D1(size) => Extent3D { width: size, height: 1, depth: 1, },
            Kind::D2 { width, height, } => Extent3D { width, height, depth: 1, },
            Kind::D3 { width, height, depth } => Extent3D { width, height, depth, },
        }
    }
}

pub struct Image;



