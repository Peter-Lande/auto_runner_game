use bevy::{prelude::*, sprite::Anchor, time::Stopwatch};
use rand::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::rgb(
            245. / 255.,
            245. / 255.,
            201. / 255.,
        )))
        .insert_resource(ObstacleOnScreen(false))
        .insert_resource(ScoreStopwatch(Stopwatch::new()))
        .add_startup_system(initialization)
        .add_system(score)
        .add_system(player_jump)
        .add_system(obstacle_movement)
        .add_system(keyboard_input)
        .run();
}

struct ObstacleOnScreen(bool);

struct ScoreStopwatch(Stopwatch);

struct ScoreFont(Handle<Font>);

#[derive(Component)]
enum Jumping {
    None,
    Jump(f32),
}

#[derive(Component)]
struct Obstacle {
    moving: bool,
    delay: Timer,
    delay_start: f32,
    delay_end: f32,
}

#[derive(Component)]
struct Scoreboard;

fn initialization(
    mut commands: Commands,
    mut windows: ResMut<Windows>,
    asset_server: Res<AssetServer>,
) {
    let window = windows.primary_mut();
    window.set_resizable(false);
    window.set_title("Auto Runner".to_string());
    let right_edge = window.width() / 2.;
    let font_handle: Handle<Font> = asset_server.load("fonts/PixelEmulator.ttf");
    let text_style = TextStyle {
        font: font_handle.as_weak(),
        font_size: 20.0,
        color: Color::BLACK,
    };
    commands.spawn_bundle(Camera2dBundle::default());
    //Background needs to be on bottom layer
    commands.spawn_bundle(SpriteBundle {
        texture: asset_server.load("textures/background.png"),
        sprite: Sprite {
            custom_size: Some(Vec2::new(window.width(), window.height())),
            ..default()
        },
        ..default()
    });

    //Character needs to be on top layer
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(1., 0., 0.),
                custom_size: Some(Vec2::new(50., 100.)),
                anchor: Anchor::BottomCenter,
                ..default()
            },
            transform: Transform::from_xyz(-500., -250., 3.),
            ..default()
        })
        .insert(Jumping::None);
    //Obstacle needs to be below top layer but not in background
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0., 1., 0.),
                custom_size: Some(Vec2::new(100., 50.)),
                anchor: Anchor::BottomCenter,
                ..default()
            },
            transform: Transform::from_xyz(right_edge + 50., -250., 2.),
            ..default()
        })
        .insert(Obstacle {
            moving: false,
            delay: Timer::from_seconds(1.0, false),
            delay_start: 1.,
            delay_end: 3.,
        });
    //Obstacle needs to be below top layer but not in background
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0., 1., 0.),
                custom_size: Some(Vec2::new(50., 75.)),
                anchor: Anchor::BottomCenter,
                ..default()
            },
            transform: Transform::from_xyz(right_edge + 25., -250., 2.),
            ..default()
        })
        .insert(Obstacle {
            moving: false,
            delay: Timer::from_seconds(3.0, false),
            delay_start: 3.0,
            delay_end: 6.0,
        });
    //Needs to be on top most layer
    commands
        .spawn_bundle(Text2dBundle {
            text: Text::from_sections(vec![
                TextSection::new("Score\n", text_style.clone()),
                TextSection::new("00000", text_style.clone()),
            ])
            .with_alignment(TextAlignment::CENTER),
            transform: Transform::from_xyz(0., (window.height() / 2.) - 20., 3.),
            ..default()
        })
        .insert(Scoreboard);
    commands.insert_resource(ScoreFont(font_handle));
}

fn score(
    time: Res<Time>,
    font: Res<ScoreFont>,
    mut score_stopwatch: ResMut<ScoreStopwatch>,
    mut text_entity: Query<&mut Text, With<Scoreboard>>,
) {
    score_stopwatch.0.tick(time.delta());
    let score: u16 = (score_stopwatch.0.elapsed_secs() / 0.1) as u16;
    let score_pretext = score.to_string();
    let score_text = "0".repeat(5 - score_pretext.len()) + &score_pretext;
    let mut scoreboard_text = text_entity.single_mut();
    let text_style = TextStyle {
        font: font.0.as_weak(),
        font_size: 20.0,
        color: Color::BLACK,
    };
    scoreboard_text.sections[1] = TextSection::new(score_text, text_style);
}

fn player_jump(
    time: Res<Time>,
    mut player_position: Query<(&mut Jumping, &mut Transform), With<Jumping>>,
) {
    for (mut jump_state, mut transform) in &mut player_position {
        match *jump_state {
            Jumping::None => (),
            Jumping::Jump(velocity) => {
                transform.translation.y +=
                    velocity * time.delta_seconds() - 400. * time.delta_seconds().powf(2.);
                *jump_state = Jumping::Jump(velocity - 800. * time.delta_seconds());
            }
        }
        if transform.translation.y <= -250. {
            *jump_state = Jumping::None;
            transform.translation.y = -250.;
        }
    }
}

fn obstacle_movement(
    time: Res<Time>,
    windows: Res<Windows>,
    mut on_screen: ResMut<ObstacleOnScreen>,
    score_stopwatch: Res<ScoreStopwatch>,
    mut obstacle_position: Query<(&mut Transform, &Sprite, &mut Obstacle), With<Obstacle>>,
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
                obstacle.delay = Timer::from_seconds(delay, false);
                on_screen.0 = false;
            } else {
                let exponent: u16 = (score_stopwatch.0.elapsed_secs() / 0.9) as u16;
                let velocity = 300. * 1.01_f32.powf(exponent as f32);
                info!(velocity);
                transform.translation.x -= velocity * time.delta_seconds();
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
                    *jump_state = Jumping::Jump(600.);
                }
            }
            _ => (),
        }
    }
}
