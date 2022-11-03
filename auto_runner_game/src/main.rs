use bevy::{prelude::*, sprite::Anchor};
use rand::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ObstacleOnScreen(false))
        .add_startup_system(initialization)
        .add_system(player_jump)
        .add_system(obstacle_movement)
        .add_system(keyboard_input)
        .run();
}

struct ObstacleOnScreen(bool);

#[derive(Component)]
enum Jumping {
    None,
    Up(f32),
    Down(f32),
}

#[derive(Component)]
struct Obstacle {
    moving: bool,
    delay: Timer,
    delay_start: f32,
    delay_end: f32,
}

fn initialization(mut commands: Commands, mut windows: ResMut<Windows>) {
    let window = windows.primary_mut();
    window.set_resizable(false);
    let right_edge = window.width() / 2.;
    commands.spawn_bundle(Camera2dBundle::default());
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(1.0, 0.0, 0.0),
                custom_size: Some(Vec2::new(50.0, 100.0)),
                anchor: Anchor::BottomCenter,
                ..default()
            },
            transform: Transform::from_xyz(-500.0, -250.0, 0.0),
            ..default()
        })
        .insert(Jumping::None);
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0., 1., 0.),
                custom_size: Some(Vec2::new(100., 50.)),
                anchor: Anchor::BottomCenter,
                ..default()
            },
            transform: Transform::from_xyz(right_edge + 50., -250., 0.),
            ..default()
        })
        .insert(Obstacle {
            moving: false,
            delay: Timer::from_seconds(1.0, false),
            delay_start: 1.0,
            delay_end: 3.0,
        });
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0., 1., 0.),
                custom_size: Some(Vec2::new(50., 75.)),
                anchor: Anchor::BottomCenter,
                ..default()
            },
            transform: Transform::from_xyz(right_edge + 25., -250., 0.),
            ..default()
        })
        .insert(Obstacle {
            moving: false,
            delay: Timer::from_seconds(3.0, false),
            delay_start: 3.0,
            delay_end: 6.0,
        });
}

fn player_jump(
    time: Res<Time>,
    mut player_position: Query<(&mut Jumping, &mut Transform), With<Jumping>>,
) {
    for (mut jump_state, mut transform) in &mut player_position {
        match *jump_state {
            Jumping::None => (),
            Jumping::Up(velocity) => {
                transform.translation.y += velocity * time.delta_seconds();
                *jump_state = Jumping::Up(velocity - 200. * time.delta_seconds());
            }
            Jumping::Down(velocity) => {
                transform.translation.y -= velocity * time.delta_seconds();
                *jump_state = Jumping::Down(velocity + 200. * time.delta_seconds());
            }
        }
        if transform.translation.y > -50. {
            match *jump_state {
                Jumping::Up(velocity) => *jump_state = Jumping::Down(velocity),
                _ => (),
            }
        } else if transform.translation.y <= -250. {
            *jump_state = Jumping::None;
            transform.translation.y = -250.;
        }
    }
}

fn obstacle_movement(
    time: Res<Time>,
    windows: Res<Windows>,
    mut on_screen: ResMut<ObstacleOnScreen>,
    mut obstacle_position: Query<(&mut Transform, &mut Sprite, &mut Obstacle), With<Obstacle>>,
) {
    for (mut transform, sprite, mut obstacle) in &mut obstacle_position {
        if obstacle.moving {
            let window = windows.primary();
            let window_edge = window.width() / 2.;
            let sprite_edge = sprite.custom_size.unwrap_or_default().x / 2.;
            if transform.translation.x < -(window_edge + sprite_edge) {
                transform.translation.x = window_edge + sprite_edge;
                obstacle.moving = false;
                let mut rng = rand::thread_rng();
                let delay: f32 = rng.gen_range(obstacle.delay_start..obstacle.delay_end);
                info!(delay);
                obstacle.delay = Timer::from_seconds(delay, false);
                on_screen.0 = false;
            } else {
                transform.translation.x -= 300. * time.delta_seconds();
            }
        } else if !on_screen.0 && obstacle.delay.tick(time.delta()).just_finished() {
            obstacle.moving = true;
            on_screen.0 = true;
        }
    }
}

fn keyboard_input(keyboard_input: Res<Input<KeyCode>>, mut sprite_jump_state: Query<&mut Jumping>) {
    for mut jump_state in &mut sprite_jump_state {
        match *jump_state {
            Jumping::None => {
                if keyboard_input.just_pressed(KeyCode::Space) {
                    *jump_state = Jumping::Up(400.);
                }
            }
            _ => (),
        }
    }
}
