mod particles;
mod mathvec;
mod physconstants;
mod gridhandler;
mod masstree;
mod physics_handler;
mod particleman;
mod easytime; 
mod pointsdl;
mod camera;

use particleman::ParticleManager;
use mathvec::{Vec2d, Scalar};
use physics_handler::PhysicsHandler;
use physics_handler::threaded::PhysicsHandlerThreaded;
use pointsdl::Screen;

use std::time::{Instant};


fn main() -> Result<(), String> {

    let nmult = 3.0; 
    let mut phys = PhysicsHandlerThreaded::new(2.0 * nmult, 1000.0, 10.0);
    let mut _start = Instant::now();
    let mut _end = Instant::now();
    let _timer = easytime::EasyTimer::now();

    let mut man = ParticleManager::new().with_mass(20.0);
    man.place_gaussian(100, 100.0, 2.0, 0.0000002);

    man = man
        .with_pos(Vec2d::new(-1000.0, 0.0))
        .with_vel(Vec2d::new(20.0, 0.0))
        .with_mass(10.0);
    man.place_ball(1600, 0.7, 0.02);

    man = man
        .with_mass(20.0)
        .with_vel(Vec2d::new(3.0, 0.0))
        .with_pos(Vec2d::new(-1000.0, 180.0));
    man.place_ball(100, 1.0, 0.0);

    man = man
        .with_pos(Vec2d::new(4000.0, 0.0))
        .with_vel(Vec2d::new(-100.0, 0.0));
    man.place_ball(500, 1.0, -0.01);

    phys.load_particles(man.into_inner());

    let _frame = 0;
    
    let mut n = 0;
    let mut _create_total = 0.0;
    let mut _phy_total = 0.0;
    let mut _delete_total = 0.0;
    let _zoom_particle = 0;

    let mut screen = Screen::init()?;
    screen.draw_context.init_draw_style_points()?;
    screen.draw_context.init_draw_style_C_circles()?;
    screen.draw_context.set_color(0,0,255,200);
    phys.zero_momentum_and_cm();
    let _a0 = phys.angular_momentum();
    while screen.should_loop() {
        for _ in 0..200 {
            phys.update(0.01);
        }
        screen.draw_context.init_draw_style_C_circles()?;
        let mult = screen.cam.scale();
        screen.draw_context.renderer.set_draw_color((0,0,0,255));
        screen.draw_context.renderer.clear();
        let transx = screen.cam.x();
        let transy = screen.cam.y();
        screen.draw_context.set_size((mult * 10.0) as u32);

        for part in phys.particles() {
            screen.draw_context.draw_point(
                (part.pos.x - transx).floor() as i32, 
                (part.pos.y - transy).floor() as i32
            )?;
        }
        screen.draw_context.renderer.set_draw_color((255, 0, 0, 255));
        screen.draw_context.renderer.draw_point(((screen.scwidth/2) as i32, (screen.scheight/2) as i32))?;
        screen.sync();
        n += 1;
        screen.sync();

    }
    screen.quit();
    Ok(())
}
