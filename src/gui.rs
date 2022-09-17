use std::ops::IndexMut;

use imgui::*;
use imgui_wgpu::{Renderer, RendererConfig};
use winit::window::Window;

use crate::gpu::Gpu;

const BROWN: [f32; 4] = [0.45, 0.22, 0.15, 1.0];
const BLUE: [f32; 4] = [0.16, 0.56, 0.69, 1.0];

pub struct Gui {
    pub context: imgui::Context,
    pub renderer: Renderer,
    pub platform: imgui_winit_support::WinitPlatform,
}
impl Gui {
    pub fn new(window: &Window, gpu: &Gpu) -> Self {
        // Set up dear imgui
        let mut imgui = imgui::Context::create();
        let mut platform = imgui_winit_support::WinitPlatform::init(&mut imgui);
        platform.attach_window(
            imgui.io_mut(),
            &window,
            imgui_winit_support::HiDpiMode::Default,
        );
        imgui.set_ini_filename(None);

        let hidpi_factor = window.scale_factor();
        let font_size = (15.0 * hidpi_factor) as f32;
        imgui.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;

        imgui.fonts().add_font(
            &[FontSource::TtfData {
                data: include_bytes!("../font/Oxanium/static/Oxanium-Light.ttf"),
                size_pixels: font_size,
                config: Some(imgui::FontConfig {
                    oversample_h: 4,
                    oversample_v: 4,
                    rasterizer_multiply: 1.5,
                    ..Default::default()
                }),
            }]
        );

        let style = imgui.style_mut();
        *style.index_mut(StyleColor::Text) = [1.0, 0.9, 1.0, 1.0];
        *style.index_mut(StyleColor::Border) = BLUE;

        *style.index_mut(StyleColor::FrameBg) = BLUE;
        *style.index_mut(StyleColor::FrameBgHovered) = BROWN;
        *style.index_mut(StyleColor::FrameBgActive) = BROWN;

        *style.index_mut(StyleColor::Button) = BLUE;
        *style.index_mut(StyleColor::ButtonHovered) = BROWN;
        *style.index_mut(StyleColor::ButtonActive) = BROWN;

        *style.index_mut(StyleColor::TitleBg) = BLUE;
        *style.index_mut(StyleColor::TitleBgActive) = BLUE;
        *style.index_mut(StyleColor::TitleBgCollapsed) = BLUE;

        *style.index_mut(StyleColor::Header) = BLUE;
        *style.index_mut(StyleColor::HeaderHovered) = BROWN;
        *style.index_mut(StyleColor::HeaderActive) = BROWN;

        *style.index_mut(StyleColor::ResizeGrip) = BLUE;
        *style.index_mut(StyleColor::ResizeGripHovered) = BROWN;
        *style.index_mut(StyleColor::ResizeGripActive) = BROWN;

        *style.index_mut(StyleColor::Tab) = BLUE;
        *style.index_mut(StyleColor::TabHovered) = BROWN;
        *style.index_mut(StyleColor::TabActive) = BROWN;

        *style.index_mut(StyleColor::NavHighlight) = BROWN;


        let renderer_config = RendererConfig {
            texture_format: gpu.config.format,
            ..Default::default()
        };

        let renderer = Renderer::new(&mut imgui, &gpu.device, &gpu.queue, renderer_config);

        Self {
            context: imgui,
            platform,
            renderer,
        }
    }
}


