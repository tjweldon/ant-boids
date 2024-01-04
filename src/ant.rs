use crate::signals::{Signals, Qualia};
use bevy::prelude::*;
use rand::prelude::*;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum SignalKind {
    Exploring,
    Retrieving,
}

#[derive(Debug)]
pub struct Ant {
    pub state: SignalKind,
    pub position: Vec2,
    pub velocity: Vec2,
}

impl Ant {
    const MAX_SPEED: f32 = 100f32;

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

    pub fn percieve_signals(&self, signals: &Signals<SignalKind, Vec2>) -> (f32, f32) {
        let sample: Qualia<SignalKind, Vec2> = signals.sample(self.position);
        let mut steering: f32 = 0.0;
        let mut gas: f32 = 0.0;
        let qualia = sample.get_qualia();
        if qualia.len() == 0 {
            return self.random_walk();
        }
        let normalizer = qualia.len() as f32;

        for qual in qualia {
            let kind = qual.kind;

            if self.state == kind || qual.amount.length() < 0.01 || qual.amount.is_nan() {
                let (g, s) = self.random_walk();
                gas += g;
                steering += s;
            } else {
                let desired_direction = - qual.amount.normalize_or_zero();
                let angle_needed = self.velocity.angle_between(desired_direction);
                if f32::is_nan(angle_needed) {
                    println!("desired_direction: {desired_direction:?}");
                    panic!();
                }
                let sign = angle_needed / angle_needed.abs();
                steering += sign * angle_needed.abs().min(90.0);
                gas += self.velocity.normalize().dot(desired_direction).min(0.0);
            }
            if f32::is_nan(gas) || f32::is_nan(steering) {
                println!("gas: {gas:#?}");
                println!("steer: {steering:#?}");
                println!("{qual:#?}");
                panic!();
            }
        }

        let motion = (gas/normalizer, steering/normalizer);

        return motion;
    }

    pub fn leave_signal(&self, signals: &mut Signals<SignalKind, Vec2>) {
        signals.leave_signal(self.position, self.velocity.normalize(), self.state)
    }

    pub fn update(&mut self, signals: &mut Signals<SignalKind, Vec2>, dt: f32) {
        let (gas, steer) = self.percieve_signals(signals);
        let new_heading = Vec2::from_angle(dt * steer);
        self.velocity = self.velocity.rotate(new_heading).normalize();
        self.position += Self::MAX_SPEED * gas * dt * self.velocity;
        self.leave_signal(signals);
    }
}
