use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(initialization)
        .add_system(sprite_jump)
        .add_system(keyboard_input)
        .run();
}

#[derive(Component)]
enum Jumping {
    None,
    Up(f32),
    Down(f32),
}

fn initialization(mut commands: Commands) {
    commands.spawn_bundle(Camera2dBundle::default());
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(1.0, 0.0, 0.0),
                custom_size: Some(Vec2::new(50.0, 100.0)),
                ..default()
            },
            transform: Transform::from_xyz(-500.0, -250.0, 0.0),
            ..default()
        })
        .insert(Jumping::None);
}

fn sprite_jump(time: Res<Time>, mut sprite_position: Query<(&mut Jumping, &mut Transform)>) {
    for (mut jump_state, mut transform) in &mut sprite_position {
        match *jump_state {
            Jumping::None => (),
            Jumping::Up(velocity) => {
                info!(velocity);
                transform.translation.y += velocity * time.delta_seconds();
                *jump_state = Jumping::Up(velocity - 400. * time.delta_seconds());
            }
            Jumping::Down(velocity) => {
                info!(velocity);
                transform.translation.y -= velocity * time.delta_seconds();
                *jump_state = Jumping::Down(velocity + 400. * time.delta_seconds());
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

fn keyboard_input(keyboard_input: Res<Input<KeyCode>>, mut sprite_jump_state: Query<&mut Jumping>) {
    for mut jump_state in &mut sprite_jump_state {
        match *jump_state {
            Jumping::None => {
                if keyboard_input.just_pressed(KeyCode::Space) {
                    *jump_state = Jumping::Up(600.);
                }
            }
            _ => (),
        }
    }
}
