use sdl2::{self, EventPump, Sdl};
use sdl2::event::{Event};
use sdl2::keyboard::{Keycode};
use sdl2::mouse::{MouseButton};
use sdl2::video::{Window, WindowContext, GLContext};
use sdl2::image::{self, InitFlag, Sdl2ImageContext, LoadSurface, LoadTexture};
use sdl2::render::{BlendMode, Canvas};
use sdl2::surface::{Surface};
use sdl2::render::Texture as SDL_Texture;
use sdl2::pixels::{PixelMasks, PixelFormatEnum};
use sdl2::rect::{Rect};

use crate::camera::CameraController;

use std::path::Path;

const DEFAULT_WIDTH : u32 = 800;
const DEFAULT_HEIGHT : u32 = 800;



pub struct Screen {
    pub scwidth : u32, 
    pub scheight : u32,
    sdl_ctx : Sdl,
    pub cam : CameraController,
    event_pump : EventPump,
    pub image_context : Sdl2ImageContext,
    running : bool, 
    mouse_context : MouseInfo,
    pub draw_context : ParticleDrawingContext,
}

#[derive(Copy, Clone, Default)]
pub struct MouseInfo {
    down : bool, 
    ndcmousex : f64, 
    ndcmousey : f64, 
}

impl Screen {
    pub fn init() -> Result<Self, String> {
        let cam = CameraController::new(0.0, 0.0,0.0, (DEFAULT_HEIGHT as f64)/(DEFAULT_WIDTH as f64));
        let sdl_ctx = sdl2::init()?;
        let events = sdl_ctx.event_pump()?;
        let video_subsystem = sdl_ctx.video()?;
        if !sdl2::hint::set("SDL_RENDER_SCALE_QUALITY", "1") {
            eprintln!("Warning: Linear texture filtering not enabled!");
        }
        let window : Window = video_subsystem.window("Rust threaded nbody", DEFAULT_WIDTH, DEFAULT_HEIGHT)
            .build()
            .map_err(|e| format!("{:?}", e))?;
        let mut renderer = window.into_canvas().accelerated().build().map_err(|e| format!("{:?}", e))?; 
        renderer.set_draw_color((0xFF, 0xFF, 0xFF, 0xFF));
        let flag = InitFlag::PNG;
        let image_ctx = image::init(flag)?;
        renderer.set_blend_mode(BlendMode::Add);
        Ok(Screen {
            scwidth : DEFAULT_WIDTH,
            scheight : DEFAULT_HEIGHT,
            sdl_ctx,
            cam,
            event_pump : events,
            image_context : image_ctx,
            running : true, 
            mouse_context : MouseInfo::default(),
            draw_context : ParticleDrawingContext::new(renderer),
        })
    }

    pub fn should_loop(&mut self) -> bool {
        self.handle_events();
        self.running
    }

    pub fn sync(&mut self) {
        self.draw_context.renderer.present();
    }

    fn handle_events(&mut self) {
        let mut mousezoom = 0;
        while let Some(e) = self.event_pump.poll_event() {
            match e {
                Event::Quit{..} | Event::KeyDown {keycode : Some(Keycode::Escape), ..} => {
                    self.running = false;
                },
                Event::MouseButtonDown{mouse_btn : MouseButton::Left, x, y, ..} => {
                    self.mouse_context.down = true;
                    self.mouse_context.ndcmousex = x as f64;
                    self.mouse_context.ndcmousey = y as f64;
                },
                Event::MouseWheel{y, ..} => {
                    mousezoom += y;
                },
                Event::MouseButtonUp{mouse_btn : MouseButton::Left, ..} => {
                    self.mouse_context.down = false;
                    self.cam.end_drag();
                },
                Event::MouseMotion{x, y, ..} => {
                    self.mouse_context.ndcmousex = x as f64;
                    self.mouse_context.ndcmousey = y as f64;
                },
                _ => {}
            }
        }
        if mousezoom != 0 {
            self.cam.mouse_zoom(mousezoom as f64);
            self.cam.set_mouse_zoom_pos(self.mouse_context.ndcmousex, self.mouse_context.ndcmousey);
        }
        if self.mouse_context.down {
            self.cam.drag(self.mouse_context.ndcmousex, self.mouse_context.ndcmousey);
        }
        self.cam.update(1.0/30.0);
    }

    pub fn quit(mut self) {

    }

    pub fn save_screenshot_bmp<T : AsRef<Path>>(filepath : T, renderer : Canvas<Window>) -> Result<(), String> {
        let format = PixelFormatEnum::ARGB8888;
        let pixels = renderer.read_pixels(None, format)?;

        let mut surface = sdl2::surface::Surface::from_pixelmasks(DEFAULT_WIDTH, DEFAULT_HEIGHT, format.into_masks()?)?;
        let raw_surface = surface.raw();
        let raw_surface_ref = unsafe { raw_surface.as_mut().ok_or("Error getting raw output surface!")?};
        Ok(())
    }
}

pub struct ParticleDrawingContext {
    pub renderer : Canvas<Window>,
    ptype : u8, 
    size : u32, 
    color : (u8, u8, u8, u8),
    i : u32, 
    particles : Vec<LTexture>,
}

impl ParticleDrawingContext {

    pub fn new(renderer : Canvas<Window>) -> ParticleDrawingContext {
        ParticleDrawingContext {
            renderer,
            ptype : 0,
            size : 0,
            color : (0, 0, 0, 0),
            i : 0,
            particles : Vec::with_capacity(4),
        }
    }

    pub fn init_draw_style_points(&mut self) -> Result<(), String> {
        self.ptype = 0;
        self.size = 0;
        self.color = (0, 0, 0, 255);
        self.particles.clear();
        self.i = 0;
        Ok(())
    }

    pub fn init_draw_style_circles(&mut self) -> Result<(), String> {
        self.ptype = 1;
        self.color = (0, 0, 0, 255);
        self.size = 0;
        self.particles.clear();
        let mut texture = LTexture::default();
        texture.load_from_file(&mut self.renderer, "particle1.png")?;
        self.particles.push(texture);
        self.i = 0;
        Ok(())
    }
    pub fn init_draw_style_T_circles(&mut self) -> Result<(), String> {
        self.ptype = 2;
        self.color = (0, 0, 0, 255);
        self.size = 0;
        self.particles.clear();
        let mut texture = LTexture::default();
        texture.load_from_file(&mut self.renderer, "particle.png")?;
        self.particles.push(texture);
        self.i = 0;
        Ok(())
    }
    pub fn init_draw_style_C_circles(&mut self) -> Result<(), String> {
        self.ptype = 3;
        self.color = (0, 0, 0, 255);
        self.size = 0;
        self.particles.clear();
        for idx in 1..5 {
            let mut texture = LTexture::default();
            texture.load_from_file(&mut self.renderer, format!("particle{}.png", idx))?;
            self.particles.push(texture);
        }
        self.i = 0;
        Ok(())
    }

    pub fn set_color(&mut self, r : u8, b : u8, g : u8, a : u8) {
        self.color = (r, b, g, a);
    }
    pub fn set_size(&mut self, s : u32) {
        self.size = s;
    }

    pub fn draw_point(&mut self, x : i32, y : i32) -> Result<(), String> {
        match self.ptype {
            0 => {
                self.renderer.set_draw_color(self.color);
                self.renderer.draw_point((x, y))?;
            },
            1 => {
                let texture = self.particles.get_mut(0).ok_or("Error: ptype = 1 but no textures loaded!")?;
                texture.render_wh(&mut self.renderer, x, y, self.size, self.size)?;
            },
            2 => {
                let texture = self.particles.get_mut(0).ok_or("Error: ptype = 2 but no textures loaded!")?;
                texture.render_wh(&mut self.renderer, x, y, self.size, self.size)?;
            },
            3 => {
                let idx = (self.i % 4) as usize;
                let texture = self.particles.get_mut(idx).ok_or(format!("Error: ptype = 3 but texture {} is not loaded!", idx))?;
                texture.render_wh(&mut self.renderer, x, y, self.size, self.size)?;
                self.i += 1;
            },
            _ => {}
        }
        Ok(())
    }
}

pub struct LTexture{
    mwidth : u32, 
    mheight : u32, 
    mtexture : Option<SDL_Texture>,
}

impl LTexture {

    pub fn width(&self) -> u32 {
        self.mwidth
    }
    pub fn height(&self) -> u32 {
        self.mheight
    }

    fn render_inner(&mut self, renderer : &mut Canvas<Window>, x : i32, y : i32, w : Option<u32>, h : Option<u32>) -> Result<(), String> {
        let texture = self.mtexture.as_ref().ok_or("Error: texture not yet initialized.")?;
        let dst = match (w, h) {
            (Some(w), Some(h)) => {
                Rect::new(x, y, w, h)
            },
            _ => {
                Rect::new(x, y, self.width(), self.height())
            }
        };

        renderer.copy(&texture, None, dst)
    }

    pub fn render_wh(&mut self, renderer : &mut Canvas<Window>, x : i32, y : i32, w : u32, h : u32) -> Result<(), String> {
        self.render_inner(renderer, x, y, Some(w), Some(h))
    }
    pub fn load_from_file<T : AsRef<Path>>(&mut self, renderer : &mut Canvas<Window>, path : T) -> Result<(), String> {
        let mut loaded_surface : Surface = Surface::from_file(path)?;
        loaded_surface.set_color_key(true, (0, 0xFF, 0xFF).into())?;
        let mwidth = loaded_surface.width();
        let mheight = loaded_surface.height();
        let new_texture = renderer.texture_creator().create_texture_from_surface(loaded_surface).map_err(|e| format!("{:?}", e))?;

        self.mwidth = mwidth;
        self.mheight = mheight;
        self.mtexture = Some(new_texture);
        Ok(())
    }
}

impl Drop for LTexture {
    fn drop(&mut self) {
        if let Some(t) = self.mtexture.take() {
            unsafe {
                t.destroy();
            }
        }
    }
}

impl Default for LTexture {
    fn default() -> LTexture {
        LTexture {
            mwidth : 0,
            mheight : 0,
            mtexture : None,
        }
    }
}