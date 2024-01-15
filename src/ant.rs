use bevy::prelude::*;
use rand::prelude::*;

use crate::field::Signals;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Component)]
#[repr(u8)]
pub enum SignalKind {
    Exploring,
    Retrieving,
}

#[derive(Debug, Component)]
pub struct Ant {
    pub state: SignalKind,
    pub position: Vec2,
    pub velocity: Vec2,
}

impl Ant {
    const MAX_SPEED: f32 = 200f32;

    pub fn new() -> Self {
        Self {
            state: SignalKind::Exploring,
            position: Vec2::ZERO,
            velocity: Vec2::X,
        }
    }

    fn random_walk(&self) -> (f32, f32) {
        let mut rng = rand::thread_rng();
        let steer: f32 = rng.gen::<f32>() * 180.0f32.to_radians() - 90.0f32.to_radians();
        return (1.0, 2.0*steer);
    }

    fn follow(&self, direction: Vec2) -> (f32, f32) {
        let desired_direction = direction.normalize_or_zero();
        let (mut gas, mut steering) = (0., 0.);
        
        if desired_direction != Vec2::ZERO && !desired_direction.is_nan() {
            let heading = match self.velocity.normalize_or_zero().is_normalized() {
                false => desired_direction.clone(),
                _ => self.velocity.normalize(),
            };

            let angle_needed = heading.angle_between(desired_direction);
            if angle_needed != 0.0 {
                let sign = angle_needed / angle_needed.abs();
                steering += sign * angle_needed.abs().min(90.0f32.to_radians());
            }
            gas = heading.dot(desired_direction).min(0.1).max(1.0);
        }
        return (gas, steering);
    }

    pub fn percieve_signals(&self, signals: &Signals) -> (f32, f32) {
        let mut weights = [20.0, 0.0];
        let (g, s) = self.random_walk();

        let mut gasses = [g, 0.0];
        let mut steers = [s, 0.0];
        
        let exploring_sig = signals.get_field(SignalKind::Exploring).sample(self.position);
        let retrieving_sig = signals.get_field(SignalKind::Retrieving).sample(self.position);

        match self.state {
            SignalKind::Exploring => {
                weights[1] += 20.;
                let (g, s) = self.follow(-retrieving_sig);
                gasses[1] += g;
                steers[1] += s;
            },
            SignalKind::Retrieving => {
                let desired_direction = retrieving_sig + (-exploring_sig);
                weights[1] += desired_direction.length();
                let (g, s) = self.follow(desired_direction);
                gasses[1] += g;
                steers[1] += s;
            },
        }

        let normalize: f32 = f32::powi(weights.iter().sum(), -1);
        let gas: f32 = normalize * weights.iter().zip(gasses).map(|(&w, g): (&f32, f32)| w * g).sum::<f32>();
        let steering: f32 = normalize * weights.iter().zip(steers).map(|(&w, s)| w * s).sum::<f32>();
        
        return (gas, steering);
    }


    pub fn leave_signal(&self, sigs: &mut Signals) {
        let sig = 10.0 * self.velocity.clone().normalize();
        sigs.get_mut_field(self.state)
            .acc_cell_value(sig, self.position);
    }

    pub fn update(&mut self, sigs: &Signals, &dt: &f32) {
        let (mut gas, mut steer) = self.percieve_signals(sigs);
        if gas.is_nan() {
            gas = 1.0;
        }
        if steer.is_nan() {
            steer = 0.0;
        }
        steer = steer.clamp(-180f32.to_radians(), 180.0f32.to_radians());
        
        let new_heading = (self.velocity + steer * 5.0 * dt * self.velocity.perp()).normalize();
        self.velocity = new_heading;
        self.position += Self::MAX_SPEED * gas * dt * self.velocity;
    }

    pub fn reach_around(&mut self, rect: Rect) {
        while self.position.x < rect.min.x {
            self.position.x += rect.width();
        }
        while self.position.x > rect.max.x {
            self.position.x -= rect.width();
        }
        while self.position.y < rect.min.y {
            self.position.y += rect.height();
        }
        while self.position.y > rect.max.y {
            self.position.y -= rect.height();
        }
    }
}
