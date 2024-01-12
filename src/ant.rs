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
    const MAX_SPEED: f32 = 500f32;

    pub fn new() -> Self {
        Self {
            state: SignalKind::Exploring,
            position: Vec2::ZERO,
            velocity: Vec2::X,
        }
    }

    pub fn random_walk(&self) -> (f32, f32) {
        let mut rng = rand::thread_rng();
        let steer: f32 = rng.gen::<f32>() * 180.0f32.to_radians() - 90.0f32.to_radians();
        return (1.0, steer);
    }

    pub fn percieve_signals(&self, signals: &Signals) -> (f32, f32) {
        let mut steering: f32 = 0.0;
        let mut gas: f32 = 0.0;
        let normalizer = 2f32;

        for kind in [SignalKind::Exploring, SignalKind::Retrieving] {
            let sample = signals.get_field(kind).sample(self.position);

            if self.state == kind || sample.length() < 0.01 || sample.is_nan() {
                let (g, s) = self.random_walk();
                gas += g;
                steering += s / 2.;
            } else {
                let desired_direction = sample.normalize_or_zero();

                if desired_direction == Vec2::ZERO || desired_direction.is_nan() {
                    gas += 1.;
                } else {
                    let heading = match self.velocity.normalize_or_zero().is_normalized() {
                        false => desired_direction.clone(),
                        _ => self.velocity.normalize(),
                    };

                    let angle_needed = heading.angle_between(desired_direction);
                    if angle_needed != 0.0 {
                        let sign = angle_needed / angle_needed.abs();
                        steering += 2.0 * sign * angle_needed.abs().min(90.0f32.to_radians());
                    }
                    gas += heading.dot(desired_direction).min(0.1);
                }
            }
        }

        let motion = (gas / normalizer, steering / normalizer);

        return motion;
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
