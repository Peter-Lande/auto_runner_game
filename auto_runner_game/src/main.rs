use bevy::{prelude::*, sprite::Anchor};
use rand::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(initialization)
        .add_system(player_jump)
        .add_system(obstacle_movement)
        .add_system(keyboard_input)
        .run();
}

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
                let delay: f32 = rng.gen_range(1.0..3.0);
                info!(delay);
                obstacle.delay = Timer::from_seconds(delay, false);
            } else {
                transform.translation.x -= 300. * time.delta_seconds();
            }
        } else {
            if obstacle.delay.tick(time.delta()).just_finished() {
                obstacle.moving = true;
            }
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
