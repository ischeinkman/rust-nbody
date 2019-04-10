use crate::mathvec::{Scalar, Vec2d};

pub struct CameraController {
    
    vel_drag : f64, 
    vel : Vec2d,
    camera_center : Vec2d,

    is_dragging : bool, 
    drag_start : Vec2d,
    drag_cur : Vec2d, 
    mouse_zoom_pos : Vec2d, 

    scroll_drag : f64, 
    scroll_vel : f64, 
    scroll : f64,
    scale : f64, 
    lastdt : f64, 
}

impl CameraController {

    pub fn new(xpos : Scalar, ypos : Scalar, scroll : f64, aspect : f64) -> CameraController {
        let camera_center = Vec2d::new(xpos, ypos);
        let scale = scroll.exp();
        CameraController {
            vel_drag : 0.9,
            vel : Vec2d::zero(),
            camera_center,

            is_dragging : false, 
            drag_start : Vec2d::zero(),
            drag_cur : Vec2d::zero(),
            mouse_zoom_pos : Vec2d::zero(), 


            scroll_drag : 0.87,
            scroll_vel : 0.0,
            scroll,
            scale,
            lastdt : 0.0,
        }
    }

    pub fn x(&self) -> Scalar {
        self.camera_center.x
    }
    pub fn y(&self) -> Scalar {
        self.camera_center.y
    }
    pub fn scale(&self) -> f64 {
        self.scroll.exp()
    }
    pub fn update(&mut self, timestep : Scalar) {
        
        self.lastdt = timestep;

        let x1 = self.camera_center;
        self.camera_center += self.vel * timestep;
        self.vel *= self.vel_drag;

        self.mouse_zoom_pos += self.scroll.exp() * (x1 - self.camera_center);

        let scale1 = self.scale().exp();
        self.scroll += self.scroll_vel * timestep;
        self.scroll_vel *= self.scroll_drag;
        let scale2 = self.scroll.exp();

        self.camera_center += self.mouse_zoom_pos * (1.0/scale1 - 1.0/scale2);

    }

    pub fn drag(&mut self, x : f64, y : f64) {
        if self.is_dragging {
            self.drag_start = self.drag_cur;
            self.drag_cur = Vec2d::new(x, y);
            self.camera_center += (-self.scroll).exp() * (self.drag_start - self.drag_cur);
            self.vel = Vec2d::zero(); 
        }
        else {
            self.is_dragging = true; 
            self.drag_start = Vec2d::new(x, y);
            self.drag_cur = Vec2d::new(x, y);
        }
    }
    pub fn end_drag(&mut self) {
        if self.is_dragging {
            self.is_dragging = false; 
            self.vel -= (self.drag_cur - self.drag_start)/(self.scale() * self.lastdt);
            self.drag_start = Vec2d::new(0.0, 0.0);
            self.drag_cur = Vec2d::new(0.0, 0.0);
        }
    }

    pub fn set_mouse_zoom_pos(&mut self, x : Scalar, y : Scalar) {
        self.mouse_zoom_pos = Vec2d::new(x, y);
    }

    pub fn mouse_zoom(&mut self, zamount : f64) {
        self.scroll_vel += zamount * self.scroll_drag;
    }
}