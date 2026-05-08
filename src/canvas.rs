//! Thin wrapper around libremarkable's Framebuffer for our drawing primitives.

use libremarkable::framebuffer::cgmath::{Point2, Vector2};
use libremarkable::framebuffer::common::{
    color, display_temp, dither_mode, mxcfb_rect, waveform_mode,
};
use libremarkable::framebuffer::core::Framebuffer;
use libremarkable::framebuffer::{FramebufferDraw, FramebufferRefresh, PartialRefreshMode};

pub const SCREEN_W: i32 = 1404;
pub const SCREEN_H: i32 = 1872;

pub struct Canvas {
    fb: Framebuffer,
}

impl Canvas {
    pub fn new() -> Self {
        Self {
            fb: Framebuffer::new(),
        }
    }

    pub fn clear(&mut self) {
        self.fb.clear();
    }

    pub fn full_refresh(&mut self) {
        self.fb.full_refresh(
            waveform_mode::WAVEFORM_MODE_GC16,
            display_temp::TEMP_USE_REMARKABLE_DRAW,
            dither_mode::EPDC_FLAG_USE_DITHERING_PASSTHROUGH,
            0,
            true,
        );
    }

    pub fn partial_refresh(&mut self, region: mxcfb_rect) {
        self.fb.partial_refresh(
            &region,
            PartialRefreshMode::Async,
            waveform_mode::WAVEFORM_MODE_GC16_FAST,
            display_temp::TEMP_USE_REMARKABLE_DRAW,
            dither_mode::EPDC_FLAG_USE_REMARKABLE_DITHER,
            0,
            false,
        );
    }

    pub fn draw_text(&mut self, x: i32, y: i32, text: &str, size: f32) -> mxcfb_rect {
        self.fb.draw_text(
            Point2 {
                x: x as f32,
                y: y as f32,
            },
            text,
            size,
            color::BLACK,
            false,
        )
    }

    pub fn draw_line(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, width: u32) {
        self.fb.draw_line(
            Point2 { x: x1, y: y1 },
            Point2 { x: x2, y: y2 },
            width,
            color::BLACK,
        );
    }

    pub fn fill_circle(&mut self, x: i32, y: i32, radius: u32, c: color) {
        self.fb.fill_circle(Point2 { x, y }, radius, c);
    }

    pub fn draw_circle(&mut self, x: i32, y: i32, radius: u32, c: color) {
        self.fb.draw_circle(Point2 { x, y }, radius, c);
    }

    pub fn fill_rect(&mut self, x: i32, y: i32, w: u32, h: u32, c: color) {
        self.fb.fill_rect(Point2 { x, y }, Vector2 { x: w, y: h }, c);
    }

    pub fn draw_rect(&mut self, x: i32, y: i32, w: u32, h: u32, border: u32) {
        self.fb
            .draw_rect(Point2 { x, y }, Vector2 { x: w, y: h }, border, color::BLACK);
    }
}
