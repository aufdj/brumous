use imgui::*;
use imgui_wgpu::{Renderer, RendererConfig};

use winit::window::Window;

use crate::gpu::Gpu;

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
        let font_size = (13.0 * hidpi_factor) as f32;
        imgui.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;

        imgui.fonts().add_font(&[FontSource::DefaultFontData {
            config: Some(imgui::FontConfig {
                oversample_h: 1,
                pixel_snap_h: true,
                size_pixels: font_size,
                ..Default::default()
            }),
        }]);

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


