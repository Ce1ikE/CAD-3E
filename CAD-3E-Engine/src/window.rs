use anyhow::Result;
use glutin::config::ConfigTemplateBuilder;
use glutin::context::{ContextApi, ContextAttributesBuilder, PossiblyCurrentContext};
use glutin::display::GetGlDisplay;
use glutin::prelude::*;
use glutin::surface::{Surface, WindowSurface};
use glutin_winit::DisplayBuilder;
use winit::platform::windows::WindowExtWindows;
use winit::raw_window_handle::HasRawWindowHandle;
use std::num::NonZeroU32;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window as WinitWindow, WindowAttributes};

pub struct Window {
    winit_window: Option<WinitWindow>,
    gl_surface: Option<Surface<WindowSurface>>,
    gl_context: Option<PossiblyCurrentContext>,
    width: u32,
    height: u32,
    title: String,
}

impl Window {
    pub fn new(
        width: u32, 
        height: u32, 
        title: &str,
    ) -> Self {
        Self {
            winit_window: None,
            gl_surface: None,
            gl_context: None,
            width,
            height,
            title: title.to_string(),
        }
    }

    pub fn init(&mut self, event_loop: &ActiveEventLoop) -> Result<glow::Context> {
        // the window is in charge of creating a context (OpenGL context in this case but could be Vulkan, DirectX, etc) 
        // and a surface (the surface is the part of the window where we will draw our content)
        // we use glutin and winit to create the window, context and surface
        // the context will then be handed over back to the engine to use it for rendering
        // from there on (after init) 
        // the window will only be responsible for swapping buffers (framebuffers) when provided with a new frame from the engine
        // and responding to the OS events (resize, close, keyboard, mouse, etc)
        // such that they can be forwarded to the engine for processing
        //
        // during init
        // Window  -> ctx -> Engine  
        // 
        // during rendering loop:
        // OS Events -> Window -> Engine
        // Engine -> new frame -> Window -> swap buffers

        let window_attributes = WindowAttributes::default()
            .with_title(&self.title)
            .with_inner_size(PhysicalSize::new(self.width, self.height))
            .with_resizable(true);

        let template = ConfigTemplateBuilder::new()
            .with_alpha_size(8)
            .with_depth_size(24)
            .with_stencil_size(8);

        let display_builder = DisplayBuilder::new()
            .with_window_attributes(Some(window_attributes));

        let (window, gl_config) = display_builder
            .build(event_loop, template, |configs| {
                configs
                    .reduce(|accum, config| {
                        if config.num_samples() > accum.num_samples() {
                            config
                        } else {
                            accum
                        }
                    })
                    .unwrap()
            })
            .map_err(|e| anyhow::anyhow!("Failed to create window: {}", e))?;

        let window = window.ok_or_else(|| anyhow::anyhow!("Failed to create window"))?;

        let gl_display = gl_config.display();
        let context_attributes = ContextAttributesBuilder::new().build(Some(
            window.raw_window_handle()
            .map_err(|e| anyhow::anyhow!("Failed to get window handle: {}", e))?
        ));

        let gl_context = unsafe {
            gl_display.create_context(&gl_config, &context_attributes)
            .map_err(|e| anyhow::anyhow!("Failed to create GL context: {}", e))?
        };

        let size = window.inner_size();
        let (width, height) = (
            NonZeroU32::new(size.width).unwrap(),
            NonZeroU32::new(size.height).unwrap(),
        );

        let surface_attributes = glutin::surface::SurfaceAttributesBuilder::<WindowSurface>::new()
            .build(
                window.raw_window_handle()
                .map_err(|e| anyhow::anyhow!("Failed to get window handle: {}", e))?,
                width,
                height,
            );

        let gl_surface = unsafe {
            gl_display
                .create_window_surface(&gl_config, &surface_attributes)
                .map_err(|e| anyhow::anyhow!("Failed to create window surface: {}", e))?
        };

        let gl_context = gl_context
            .make_current(&gl_surface)
            .map_err(|e| anyhow::anyhow!("Failed to make context current: {}", e))?;

        let gl = unsafe {
            glow::Context::from_loader_function(|s| {
                gl_display.get_proc_address(&std::ffi::CString::new(s).unwrap())
            })
        };

        self.winit_window = Some(window);
        self.gl_surface = Some(gl_surface);
        self.gl_context = Some(gl_context);
        self.width = size.width;
        self.height = size.height;

        Ok(gl)
    }

    pub fn swap_buffers(&self) -> Result<()> {
        
        if let (Some(surface), Some(context)) = (&self.gl_surface, &self.gl_context) {
            // windowing applications apply a double buffer for rendering
            // the Front buffer
            // the Back buffer
            // as soon as all the rendering commands are finished we swap the back buffer to the front buffer
            // so the image can be displayed without still being rendered to avoid any artifacts
            surface.swap_buffers(context)
            .map_err(|e| anyhow::anyhow!("Failed to swap buffers: {}", e))?;
        }
        
        Ok(())
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if let (Some(surface), Some(context), Some(window)) =
            (&self.gl_surface, &self.gl_context, &self.winit_window)
        {
            if let (Some(width_nz), Some(height_nz)) =
                (NonZeroU32::new(width), NonZeroU32::new(height))
            {
                surface.resize(context, width_nz, height_nz);
                self.width = width;
                self.height = height;
            }
        }
    }

    pub fn width(&self) -> u32 {
        return self.width
    }

    pub fn height(&self) -> u32 {
        return self.height
    }

    pub fn aspect_ratio(&self) -> f32 {
        return  self.width as f32 / self.height as f32
    }

    pub fn request_redraw(&self) {
        if let Some(window) = &self.winit_window {
            window.request_redraw();
        }
    }

    pub fn winit_window(&self) -> Option<&WinitWindow> {
        self.winit_window.as_ref()
    }
}
