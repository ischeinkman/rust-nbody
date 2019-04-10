
use crate::mathvec::{Vec2d, Scalar};

#[derive(Copy, Clone, PartialEq, Debug, Default)]
pub struct Particle {
    pub pos : Vec2d,
    pub vel : Vec2d,
    pub f_grav : Vec2d,
    pub f_spring : Vec2d,
    pub mass : Scalar,
    pub radius : Scalar,
}

impl Eq for Particle {}

impl Particle {
    pub fn new(mass : Scalar, radius : Scalar, pos : Vec2d, vel : Vec2d) -> Particle {
        Particle {
            pos, 
            vel,
            mass,
            radius,
            f_grav : Vec2d::zero(),
            f_spring : Vec2d::zero(), 
        }
    }
}