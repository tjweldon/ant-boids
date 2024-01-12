use crate::ant::Ant;
use crate::ant::SignalKind;
use bevy::prelude::*;
use crate::field::{Vec2Field, Vec2Cell};
use rand::rngs::ThreadRng;
use crate::field::Signals;
use rand::prelude::*;
use bevy_rand::prelude::*;
use rand_core::RngCore;
use bevy_prng::ChaCha8Rng;

const RESOLUTION: (f32, f32) = (1920f32, 1080f32);

#[derive(Component)]
struct SignalIndicator(Vec2, SignalKind);

struct Scratch;

impl Scratch {
    fn app() -> App {
        let mut app: App = App::new();
        app
            .add_plugins(DefaultPlugins)
            .add_plugins(EntropyPlugin::<ChaCha8Rng>::default())
            .insert_resource(
                Vec2Field::new(
                    SignalKind::Exploring, 
                    Vec2::splat(10.0), 
                    Vec2::new(1920.0, 1080.0),
                ),
            )
            .add_systems(Startup, Self::setup)
            .add_systems(Update, Self::diffuse_field);
        
        app
    }

    fn run() {
        Self::app().run();
    }

    fn unit_range(x: f32) -> f32 {
        let pi = 180f32.to_radians();
        (x.atan() / (0.5*pi)).powi(3) + 0.5
    }

    fn setup(
        mut commands: Commands,
        mut field: ResMut<Vec2Field>,
        mut windows: Query<&mut Window>,
        mut rng: ResMut<GlobalEntropy<ChaCha8Rng>>,
    ) {
        let mut window = windows.single_mut();
        window.resolution.set_physical_resolution(RESOLUTION.0 as u32, RESOLUTION.1 as u32);
        window.resolution.set_scale_factor(1.0);

        let mut f = || {
            let x: f32 = rng.next_u32() as f32 / u32::MAX as f32;
            return x;
        };

        field.fill_with(&mut f);
            
        field.set_cell_value(Vec2::splat(100.0), Vec2::splat(0.0));

        let lattice = field.get_lattice();
        let cells = field.get_cells();
        commands.spawn(Camera2dBundle::default());

        for &cell in cells {
            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: Color::rgb(Self::unit_range(cell.val.x), Self::unit_range(cell.val.y), 0.0),
                        rect: Some(Rect::from_corners(Vec2::ZERO, lattice.clone())),
                        ..default()
                    },
                    transform: Transform::from_translation(cell.region.center().extend(0.0)),
                    ..default()
                },
                cell
            ));
        } 
    }

    pub fn diffuse_field(
        mut query: Query<(&mut Vec2Cell, &mut Sprite), With<Transform>>,
        mut field: ResMut<Vec2Field>,
        mut rng: ResMut<GlobalEntropy<ChaCha8Rng>>,
        time: Res<Time>,
    ) {
        let threshold: f32 = 0.10f32;
        let should_drip = (rng.next_u32() as f32 / u32::MAX as f32) < threshold;
        if should_drip {
            if let Some(grid_idx) = field.get_dimensions().to_grid(
                rng.next_u32() as usize % field.get_dimensions().linear_max(),
            ) {
                let angle = 360f32.to_radians() * rng.next_u32() as f32 / u32::MAX as f32;
                field.set_cell_value_at_lattice_idx(Vec2::splat(100.0).rotate(Vec2::from_angle(angle)), grid_idx);
            }
        }

        field.update(0.6, 0.03, &time.delta_seconds());
        for (mut cell, mut sprite) in &mut query {
             cell.read_from(&field);
             sprite.color = Color::rgb(Self::unit_range(cell.val.x), Self::unit_range(cell.val.y), 0.0);
        }
    }
}
