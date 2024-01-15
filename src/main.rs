mod ant;
mod field;

use ant::Ant;
use ant::SignalKind;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_prng::ChaCha8Rng;
use bevy_rand::prelude::*;
use field::{Food, Cellular, Cell};
use field::Signals;
use field::Vec2Field;
use rand_core::RngCore;

const RESOLUTION: (f32, f32) = (1920f32, 1080f32);

#[derive(Component)]
struct Inventory {
    pub capacity: f32,
    pub contents: f32,
}

impl Inventory {
    pub fn new(capacity: f32) -> Self {
        Self {
            capacity,
            contents: 0f32,
        }
    }

    pub fn is_full(&self) -> bool {
        return self.contents > 0.5 * self.capacity;
    }

    pub fn get_space(&self) -> f32 {
        self.capacity - self.contents
    }

    pub fn fill_from(&mut self, position: Vec2, source: &mut Food) {
        let available = source.amount.get_cell_value(position.clone());
        if available > self.get_space() {
            source.amount.set_cell_value(available - self.get_space(), position.clone());
            self.contents = self.capacity;
        } else {
            source.amount.set_cell_value(0f32, position);
            self.contents += available;
        }
    }
}

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
        .insert_resource(
            Food::new(
                Vec2::splat(10.0),
                Vec2::new(RESOLUTION.0, RESOLUTION.1),
            )
        )
        .add_plugins(EntropyPlugin::<ChaCha8Rng>::default())
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, update_ants)
        .add_systems(Update, leave_signals)
        .add_systems(Update, take_food)
        .add_systems(Update, update_cells)
        .add_systems(Update, put_food)
        .run();
}

fn setup(
    mut windows: Query<&mut Window>,
    mut commands: Commands,
    mut rng: ResMut<GlobalEntropy<ChaCha8Rng>>,
    mut food: ResMut<Food>,
) {
    let mut primary_window = windows.single_mut();

    primary_window.resolution.set(RESOLUTION.0, RESOLUTION.1);
    primary_window
        .resolution
        .set_physical_resolution(RESOLUTION.0 as u32, RESOLUTION.1 as u32);
    commands.spawn(Camera2dBundle::default());
    for _ in 0..1000 {
        commands.spawn(EntityFactories::ant_factory(&mut rng));
    }
    let food_places = [
        Rect::from_center_size(Vec2::ZERO, Vec2::splat(400.)),
    ];
    let food_depth = 10.;
    for area in food_places {
        food.put(area, food_depth);
    }

    let cells = food.get_cells();
    for cell in cells {
        commands.spawn((
            cell.clone(),
            SpriteBundle {
                sprite: Sprite {
                    rect: Some(cell.region),
                    color: Color::rgb(0., 0.0, cell.val/10.0),
                    ..default()
                },
                transform: Transform::from_xyz(cell.region.center().x, cell.region.center().y, -0.1),
                ..default()
            }
        ));
    }
}

struct EntityFactories;

impl EntityFactories {
    pub fn ant_factory(
        rng: &mut ResMut<GlobalEntropy<ChaCha8Rng>>,
    ) -> (SpriteBundle, Ant, Inventory) {
        let x = 0.8 * (rng.next_u32() as f32 / u32::MAX as f32 - 0.5) * RESOLUTION.0;
        let y = 0.8 * (rng.next_u32() as f32 / u32::MAX as f32 - 0.5) * RESOLUTION.1;
        let theta = 360f32.to_radians() * (rng.next_u32() as f32 / u32::MAX as f32 - 0.5);
        let mut ant = Ant::new();
        ant.position.x = x;
        ant.position.y = y;

        ant.velocity = ant.velocity.rotate(Vec2::from_angle(theta));
        ant.state = SignalKind::Exploring;

        (
            SpriteBundle {
                sprite: Sprite {
                    color: Color::RED,
                    custom_size: Some(0.01 * Vec2::new(1920.0, 1080.0)),
                    ..default()
                },
                transform: Transform::from_translation(ant.position.extend(0f32))
                    .with_rotation(Quat::from_rotation_arc_2d(Vec2::ZERO, ant.velocity)),
                ..default()
            },
            ant,
            Inventory::new(2f32),
        )
    }
}


fn update_ants(
    mut signals: ResMut<Signals>,
    mut query: Query<(&mut Transform, &mut Ant, &mut Inventory, &mut Sprite)>,
    time: Res<Time>,
) {
    let dt = time.delta_seconds();
    signals.update(&dt);
    for (mut transform, mut ant, mut inventory, mut sprite) in &mut query {
        inventory.contents = inventory.contents - 0.1 * dt;
        inventory.contents = inventory.contents.max(0f32);
        if !inventory.is_full() {
            ant.state = SignalKind::Exploring;
            sprite.color = Color::RED;
        }
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

fn update_cells(
    mut query: Query<(&mut Cell, &mut Sprite)>,
    food: Res<Food>,
) {
    for (mut cell, mut sprite) in &mut query {
        cell.read_from(&food.amount);
        sprite.color.set_b(cell.val/10f32);
    }
}

fn leave_signals(
    mut signals: ResMut<Signals>,
    mut query: Query<&mut Ant, With<Transform>>,
) {
    for ant in &mut query {
        ant.leave_signal(&mut signals);
    }
}

fn take_food(
    mut query: Query<(&Transform, &mut Ant, &mut Inventory, &mut Sprite)>,
    mut food: ResMut<Food>,
    time: Res<Time>,
) {
    food.update(&time.delta_seconds());
    for (transform, mut ant, mut inventory, mut sprite) in &mut query {
        if ant.state == SignalKind::Exploring {
            let pos = transform.translation.truncate();
            inventory.fill_from(pos, &mut food);
            if inventory.is_full() {
                ant.state = SignalKind::Retrieving;
                sprite.color = Color::GREEN;
            }
        }
    }
}

fn put_food(
    mut food: ResMut<Food>,
    buttons: Res<Input<MouseButton>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
) {
    if buttons.pressed(MouseButton::Left) {
        if let Some(mouse_pos) = q_windows.single().cursor_position() {
            let world_pos = Vec2::new(mouse_pos.x - RESOLUTION.0/2., RESOLUTION.1/2. - mouse_pos.y) - Vec2::new(-140., 90.);
            food.put(Rect::from_center_size(world_pos, Vec2::splat(50f32)), 10f32);
        }
    }
}


