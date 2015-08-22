//! A small library that acts as a bridge between SDL2 and `gfx`, allowing `gfx`
//! to be used with SDL2 windows using OpenGL.

extern crate gfx;
extern crate gfx_device_gl;
extern crate sdl2;

pub use gfx_device_gl::{Device, Factory};

use gfx::tex::Size;
use gfx::traits::StreamFactory;
use gfx_device_gl::Resources;
use sdl2::video::{GLContext, Window};

/// The bridge between the SDL2 window and `gfx`. This implements `gfx::Output`.
pub struct Output<R: gfx::Resources> {
    /// The SDL2 window that the output was created with.
    pub window: Window,
    /// The OpenGL context for the SDL2 window. Once the `Output` is dropped
    /// the context gets destroyed (so we need to keep it around).
    pub context: GLContext,
    frame: gfx::handle::FrameBuffer<R>,
    // In `gfx_window_{glutin,glfw}` mask and gamma values are also stored.
    // Here we instead rely on the (hopefully sane) default implementation
    // of `gfx::Output::get_gamma` and just pass along a constant mask.
}

impl<R: gfx::Resources> gfx::Output<R> for Output<R> {
    fn get_handle(&self) -> Option<&gfx::handle::FrameBuffer<R>> {
        Some(&self.frame)
    }
    
    fn get_size(&self) -> (Size, Size) {
        let (w, h) = self.window.drawable_size();
        (w as Size, h as Size)
    }
    
    // Not implementing `get_mask` causes panics when calling `clear`.
    fn get_mask(&self) -> gfx::Mask {
        gfx::COLOR | gfx::DEPTH | gfx::STENCIL
    }
}

impl<R: gfx::Resources> gfx::Window<R> for Output<R> {
    fn swap_buffers(&mut self) {
        self.window.gl_swap_window();
    }
}

/// A `gfx::OwnedStream` with an OpenGL device and `Output` for output.
pub type Stream = gfx::OwnedStream<Device, Output<Resources>>;

/// Create a stream, device and factory for the given SDL2 window.
/// 
/// # Panics
/// 
/// This function will panic if an OpenGL context cannot be created for the
/// window (i.e. if `sdl2::video::Window::gl_create_context` returns `Err`).
pub fn init(window: Window) -> (Stream, Device, Factory) {
    let context = window.gl_create_context().unwrap();
    let (device, mut factory) = gfx_device_gl::create(|s| {
        window.subsystem().gl_get_proc_address(s)
    });
    let out = Output {
        window: window,
        context: context,
        frame: factory.get_main_frame_buffer(),
    };
    let stream = factory.create_stream(out);
    (stream, device, factory)
}
