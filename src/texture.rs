use std::fs;
use std::num::NonZeroU32;

use crate::error::{BrumousError, BrumousResult};

use image::GenericImageView;

pub struct Texture {
    pub texture: wgpu::Texture,
    pub view:    wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}
impl Texture {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, texture_path: Option<&str>) -> BrumousResult<Self> {
        let texture = if let Some(path) = texture_path {
            let data = fs::read(path)
                .map_err(|e| BrumousError::OpenTexture(path.to_string(), e))?;

            let img = image::load_from_memory(&data)
                .map_err(|e| BrumousError::LoadTexture(path.to_string(), e))?;
    
            let rgba = img.to_rgba8();
            let dimensions = img.dimensions();
    
            let size = wgpu::Extent3d {
                width: dimensions.0,
                height: dimensions.1,
                depth_or_array_layers: 1,
            };
            let texture = device.create_texture(
                &wgpu::TextureDescriptor {
                    label: None,
                    size,
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                }
            );
    
            queue.write_texture(
                wgpu::ImageCopyTexture {
                    aspect: wgpu::TextureAspect::All,
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                },
                &rgba,
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: NonZeroU32::new(4 * dimensions.0),
                    rows_per_image: NonZeroU32::new(dimensions.1),
                },
                size,
            );
            texture
        }
        else {
            device.create_texture(
                &wgpu::TextureDescriptor {
                    label: None,
                    size: wgpu::Extent3d {
                        width: 1,
                        height: 1,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                }
            )
        };
        
        let view = texture.create_view(
            &wgpu::TextureViewDescriptor {
                dimension: Some(wgpu::TextureViewDimension::D2),
                ..Default::default()
            }
        );
        let sampler = device.create_sampler(
            &wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Nearest,
                mipmap_filter: wgpu::FilterMode::Nearest,
                ..Default::default()
            }
        );
        
        Ok(
            Self { 
                texture, 
                view, 
                sampler,
            }
        )
    }
}