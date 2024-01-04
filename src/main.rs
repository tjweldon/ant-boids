mod ant;
mod signals;
mod enclosure;

use signals::Signals;
use ant::SignalKind;
use ant::Ant;
use bevy::prelude::*;



fn ___main() {
    let mut signals: Signals<SignalKind, Vec2> = Signals::new();
    for i in 0..5 {
        signals.update(10.0);
        let sample = signals.sample(Vec2::ZERO);
        signals.leave_signal(
            Vec2::X.rotate(Vec2::from_angle(90f32.to_radians() * i as f32)),
            (-Vec2::Y).rotate(Vec2::from_angle(90f32.to_radians() * i as f32)),
            match i % 2 { 0 => SignalKind::Exploring, _ => SignalKind::Retrieving },
        );
        println!("{sample:#?}");
    }
}


fn main() {
    let mut t = 0f32;
    let mut ant = Ant::new();
    let mut signals: Signals<SignalKind, Vec2> = Signals::new();
    signals.leave_signal(0.5 * Vec2::Y, -Vec2::Y, SignalKind::Retrieving);
    signals.update(0.1);
    t += 0.1;

    let dt = 1.0 / 60.0;
    for _ in 0..10 {
        signals.update(dt.clone());
        ant.update(&mut signals, dt.clone());
        t += dt;
        let (pos, _) = (ant.position, ant.velocity);
        print!("t={t:.3}:\t");
        println!("{pos:?}");
    }
}

fn _main() {
    let mut s: Signals<SignalKind, Vec2> = Signals::new();
    s.leave_signal(3.0 * Vec2::X, Vec2::Y, SignalKind::Exploring);
    s.leave_signal(6.0 * Vec2::X, Vec2::X, SignalKind::Exploring);
    s.update(2.0);
    let max_points: usize = 20;
    let curve: Vec<Vec2> = (0..max_points)
        .map(|i| (20.0 * i as f32 / max_points as f32 - 10.0) * Vec2::X)
        .collect();

    for (i, pt) in curve.iter().enumerate() {
        let x: f32 = 10.0 * i as f32 / max_points as f32;
        print!("{x}:\t");
        for _ in 0..(pt.dot(Vec2::splat(1.0).normalize())*30.0) as usize  {
            print!("#")
        }
        println!();
    }
}
