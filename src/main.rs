mod ant;
mod field;

use ant::Ant;
use ant::SignalKind;
use bevy::prelude::*;
use bevy_prng::ChaCha8Rng;
use bevy_rand::prelude::*;
use field::Signals;
use field::Vec2Field;
use rand_core::RngCore;

const RESOLUTION: (f32, f32) = (1920f32, 1080f32);
const INDICATOR_VISIBILITY: [(SignalKind, Visibility); 2] = [
    (SignalKind::Exploring, Visibility::Hidden),
    (SignalKind::Retrieving, Visibility::Hidden),
];

fn is_visible(kind: SignalKind) -> Visibility {
    for (cmp, vis) in INDICATOR_VISIBILITY {
        if cmp == kind {
            return vis;
        }
    }

    return Visibility::Hidden;
}

#[derive(Component)]
struct SignalIndicator(Vec2, SignalKind);

fn main() {
    sim();
}

fn sim() {
    App::new()
        .insert_resource(Signals {
            exploring: Vec2Field::new(
                SignalKind::Exploring,
                Vec2::splat(10.0),
                Vec2::new(RESOLUTION.0, RESOLUTION.1),
            ),
            retrieving: Vec2Field::new(
                SignalKind::Retrieving,
                Vec2::splat(10.0),
                Vec2::new(RESOLUTION.0, RESOLUTION.1),
            ),
        })
        .add_plugins(EntropyPlugin::<ChaCha8Rng>::default())
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, update_ants)
        .add_systems(Update, leave_signals)
        .add_systems(Update, update_indicators)
        .run();
}

fn setup(
    mut signals: ResMut<Signals>,
    mut windows: Query<&mut Window>,
    mut commands: Commands,
    mut rng: ResMut<GlobalEntropy<ChaCha8Rng>>,
) {
    let mut primary_window = windows.single_mut();

    primary_window.resolution.set(RESOLUTION.0, RESOLUTION.1);
    primary_window
        .resolution
        .set_physical_resolution(RESOLUTION.0 as u32, RESOLUTION.1 as u32);
    commands.spawn(Camera2dBundle::default());
    for i in 0..1000 {
        let kind = match i % 10 {
            0..=2 => SignalKind::Retrieving,
            _ => SignalKind::Exploring,
        };

        commands.spawn(EntityFactories::ant_factory(kind, &mut rng));

        for pos in indicator_grid() {
            commands.spawn(EntityFactories::indicator(
                pos.clone(),
                SignalKind::Exploring,
                Color::RED,
            ));
            commands.spawn(EntityFactories::indicator(
                pos.clone(),
                SignalKind::Retrieving,
                Color::GREEN,
            ));
        }
        signals.update(&1.0);
    }
}

fn indicator_grid() -> Vec<Vec2> {
    let mut places: Vec<Vec2> = vec![];
    let x_step = RESOLUTION.0 / 32f32;
    let y_step = RESOLUTION.1 / 18f32;
    for x in (-16..16).map(|xx| xx as f32 * x_step) {
        for y in (-9..9).map(|yy| yy as f32 * y_step) {
            places.push(Vec2::new(x, y))
        }
    }

    places
}

struct EntityFactories;

impl EntityFactories {
    pub fn ant_factory(
        kind: SignalKind,
        rng: &mut ResMut<GlobalEntropy<ChaCha8Rng>>,
    ) -> (SpriteBundle, Ant, SignalKind) {
        let colour = match kind {
            SignalKind::Exploring => Color::RED,
            SignalKind::Retrieving => Color::GREEN,
        };
        let x = 0.8 * (rng.next_u32() as f32 / u32::MAX as f32 - 0.5) * RESOLUTION.0;
        let y = 0.8 * (rng.next_u32() as f32 / u32::MAX as f32 - 0.5) * RESOLUTION.1;
        let theta = 360f32.to_radians() * (rng.next_u32() as f32 / u32::MAX as f32 - 0.5);
        let mut ant = Ant::new();
        ant.position.x = x;
        ant.position.y = y;

        ant.velocity = ant.velocity.rotate(Vec2::from_angle(theta));
        ant.state = kind.clone();

        (
            SpriteBundle {
                sprite: Sprite {
                    color: colour,
                    custom_size: Some(0.01 * Vec2::new(1920.0, 1080.0)),
                    ..default()
                },
                transform: Transform::from_translation(ant.position.extend(0f32))
                    .with_rotation(Quat::from_rotation_arc_2d(Vec2::ZERO, ant.velocity)),
                ..default()
            },
            ant,
            kind,
        )
    }

    pub fn indicator(
        pos: Vec2,
        kind: SignalKind,
        colour: Color,
    ) -> (SpriteBundle, SignalIndicator) {
        (
            SpriteBundle {
                sprite: Sprite {
                    color: colour,
                    rect: Some(Rect {
                        min: Vec2::splat(0.0),
                        max: Vec2::new(3., 3.),
                    }),
                    ..default()
                },
                transform: Transform::from_translation(pos.extend(0.0)),
                visibility: is_visible(kind),
                ..default()
            },
            SignalIndicator(pos, kind),
        )
    }
}

fn update_indicators(
    mut query: Query<(&mut Transform, &SignalIndicator, &Visibility)>,
    signals: Res<Signals>,
    time: Res<Time>,
) {
    for (mut transform, indicator, vis) in &mut query {
        if vis == Visibility::Hidden {
            continue;
        }
        let sample = signals.get_field(indicator.1).sample(indicator.0);
        let initial = transform.local_x().truncate();
        let norm = sample.normalize_or_zero();
        if norm == Vec2::ZERO {
            continue;
        }
        let angle = norm.angle_between(initial);
        if !f32::is_nan(angle) {
            transform.rotate(Quat::from_rotation_z(-time.delta_seconds() * angle));
        }
        transform.translation = indicator.0.extend(0.0);

        transform.scale.x = sample.length().log(2.);
        transform.scale.y = (sample.length()).min(1.);
    }
}

fn update_ants(
    mut signals: ResMut<Signals>,
    mut query: Query<(&mut Transform, &mut Ant)>,
    time: Res<Time>,
) {
    let dt = time.delta_seconds();
    signals.update(&dt);
    for (mut transform, mut ant) in &mut query {
        let old_heading = ant.velocity.normalize();
        ant.update(&mut signals, &dt);
        let res = Vec2::new(RESOLUTION.0, RESOLUTION.1);
        ant.reach_around(Rect {
            min: -0.4 * res,
            max: 0.4 * res,
        });
        if !f32::is_nan(old_heading.angle_between(ant.velocity.normalize())) {
            transform.rotation = Quat::from_rotation_arc_2d(Vec2::X, ant.velocity);
        }
        transform.translation = ant.position.extend(0.0);
    }
}

fn leave_signals(
    mut signals: ResMut<Signals>,
    mut query: Query<(&mut Ant, &SignalKind), With<Transform>>,
) {
    for (mut ant, &kind) in &mut query {
        ant.state = kind;
        ant.leave_signal(&mut signals);
        if kind == SignalKind::Retrieving {
            ant.leave_signal(&mut signals);
            ant.leave_signal(&mut signals);
        }
    }
}
