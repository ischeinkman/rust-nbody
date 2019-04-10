use crate::mathvec::Scalar;
use crate::particles::Particle;

pub mod threaded;
pub trait PhysicsHandler {
    fn new(G : Scalar, collision_spring_constant : Scalar, collision_dampening : Scalar) -> Self;
    fn zero_momentum_and_cm(&mut self);
    fn angular_momentum(&self) -> Scalar;
    fn num_particles(&self) -> usize; 
    fn load_particles<T : AsRef<[Particle]>>(&mut self, particles : T) ;
    fn update(&mut self, dt : Scalar);
}
