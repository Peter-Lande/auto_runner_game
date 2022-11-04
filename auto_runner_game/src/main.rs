use bevy::{prelude::*, sprite::Anchor, time::Stopwatch};
use rand::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_state(GameState::Menu)
        .add_startup_system(setup)
        .insert_resource(ObstacleCanSpawn(true))
        .insert_resource(ScoreStopwatch(Stopwatch::new()))
        .add_system_set(SystemSet::on_enter(GameState::Menu).with_system(initialize_menu))
        .add_system_set(SystemSet::on_update(GameState::Menu).with_system(menu))
        .add_system_set(SystemSet::on_exit(GameState::Menu).with_system(menu_cleanup))
        .add_system_set(SystemSet::on_enter(GameState::InGame).with_system(initialize_game))
        .add_system_set(
            SystemSet::on_update(GameState::InGame)
                .with_system(score)
                .with_system(player_jump)
                .with_system(obstacle_movement)
                .with_system(keyboard_input),
        )
        .run();
}

struct MenuData {
    button_entity: Entity,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum GameState {
    Menu,
    InGame,
    Paused,
}

struct ObstacleCanSpawn(bool);

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
    velocity: f32,
    delay: Timer,
    delay_start: f32,
    delay_end: f32,
}

#[derive(Component)]
struct Scoreboard;

const WINDOW_HEIGHT: f32 = 720.;
const WINDOW_WIDTH: f32 = 1280.;
const WINDOW_RIGHT: f32 = WINDOW_WIDTH / 2.;
const WINDOW_LEFT: f32 = -WINDOW_RIGHT;
const WINDOW_TOP: f32 = WINDOW_HEIGHT / 2.;

const GROUND_HEIGHT: f32 = -250.;

const BACKGROUND_COLOR: Color = Color::rgb(245. / 255., 245. / 255., 210. / 255.);
const NORMAL_BUTTON: Color = Color::BLACK;
const HOVER_BUTTON: Color = Color::GRAY;
const PRESSED_BUTTON: Color = Color::WHITE;

const ACCELERATION_PLAYER: f32 = 800.;
const INITIAL_VELOCITY_PLAYER: f32 = 600.;

const INITIAL_VELOCITY_OBSTACLE: f32 = 300.;

fn setup(mut commands: Commands, mut windows: ResMut<Windows>, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(Camera2dBundle::default());
    let window = windows.primary_mut();
    window.set_resizable(false);
    window.set_title("Auto Runner".to_string());
    window.set_resolution(WINDOW_WIDTH, WINDOW_HEIGHT);
    let font_handle: Handle<Font> = asset_server.load("fonts/PixelEmulator.ttf");
    commands.insert_resource(ScoreFont(font_handle));
    //Background needs to be on bottom layer
    commands.spawn_bundle(SpriteBundle {
        texture: asset_server.load("textures/background.png"),
        sprite: Sprite {
            custom_size: Some(Vec2::new(WINDOW_WIDTH, WINDOW_HEIGHT)),
            color: BACKGROUND_COLOR,
            ..default()
        },
        ..default()
    });
}

fn initialize_menu(mut commands: Commands, font: Res<ScoreFont>) {
    let button_entity = commands
        .spawn_bundle(ButtonBundle {
            style: Style {
                size: Size::new(Val::Px(100.), Val::Px(30.)),
                margin: UiRect::all(Val::Auto),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            color: NORMAL_BUTTON.into(),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle::from_section(
                "PLAY",
                TextStyle {
                    font: font.0.as_weak(),
                    font_size: 36.,
                    color: Color::WHITE,
                },
            ));
        })
        .id();
    commands.insert_resource(MenuData { button_entity })
}

fn menu(
    mut state: ResMut<State<GameState>>,
    mut interaction_query: Query<
        (&Interaction, &mut UiColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Clicked => {
                *color = PRESSED_BUTTON.into();
                state.set(GameState::InGame).unwrap();
            }
            Interaction::Hovered => {
                *color = HOVER_BUTTON.into();
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
            }
        }
    }
}

fn menu_cleanup(mut commands: Commands, menu_data: Res<MenuData>) {
    commands.entity(menu_data.button_entity).despawn_recursive();
}

fn initialize_game(mut commands: Commands, font: Res<ScoreFont>) {
    let text_style = TextStyle {
        font: font.0.as_weak(),
        font_size: 20.0,
        color: Color::BLACK,
    };
    //Character needs to be on top layer
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::RED,
                custom_size: Some(Vec2::new(50., 100.)),
                anchor: Anchor::BottomCenter,
                ..default()
            },
            transform: Transform::from_xyz(-500., GROUND_HEIGHT, 3.),
            ..default()
        })
        .insert(Jumping::None);
    //Obstacle needs to be below top layer but not in background
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::GREEN,
                custom_size: Some(Vec2::new(100., 50.)),
                anchor: Anchor::BottomCenter,
                ..default()
            },
            transform: Transform::from_xyz(WINDOW_RIGHT + 50., GROUND_HEIGHT, 2.),
            ..default()
        })
        .insert(Obstacle {
            moving: false,
            velocity: INITIAL_VELOCITY_OBSTACLE,
            delay: Timer::from_seconds(1.0, false),
            delay_start: 1.,
            delay_end: 3.,
        });
    //Obstacle needs to be below top layer but not in background
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::BLUE,
                custom_size: Some(Vec2::new(50., 75.)),
                anchor: Anchor::BottomCenter,
                ..default()
            },
            transform: Transform::from_xyz(WINDOW_RIGHT + 25., GROUND_HEIGHT, 2.),
            ..default()
        })
        .insert(Obstacle {
            moving: false,
            velocity: INITIAL_VELOCITY_OBSTACLE,
            delay: Timer::from_seconds(3.0, false),
            delay_start: 3.,
            delay_end: 6.,
        });
    //Needs to be on top most layer
    commands
        .spawn_bundle(Text2dBundle {
            text: Text::from_sections(vec![
                TextSection::new("Score\n", text_style.clone()),
                TextSection::new("00000", text_style),
            ])
            .with_alignment(TextAlignment::CENTER),
            transform: Transform::from_xyz(0., WINDOW_TOP - 20., 3.),
            ..default()
        })
        .insert(Scoreboard);
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
                transform.translation.y += velocity * time.delta_seconds()
                    - (ACCELERATION_PLAYER / 2.) * time.delta_seconds().powf(2.);
                *jump_state = Jumping::Jump(velocity - ACCELERATION_PLAYER * time.delta_seconds());
            }
        }
        if transform.translation.y <= GROUND_HEIGHT {
            *jump_state = Jumping::None;
            transform.translation.y = GROUND_HEIGHT;
        }
    }
}

fn obstacle_movement(
    time: Res<Time>,
    mut can_spawn: ResMut<ObstacleCanSpawn>,
    score_stopwatch: Res<ScoreStopwatch>,
    mut obstacle_position: Query<(&mut Transform, &Sprite, &mut Obstacle), With<Obstacle>>,
) {
    for (mut transform, sprite, mut obstacle) in &mut obstacle_position {
        if obstacle.moving {
            transform.translation.x -= obstacle.velocity * time.delta_seconds();
            let sprite_edge = sprite.custom_size.unwrap_or_default().x / 2.;
            if transform.translation.x < WINDOW_LEFT - sprite_edge {
                transform.translation.x = WINDOW_RIGHT + sprite_edge;
                obstacle.moving = false;
                let mut rng = rand::thread_rng();
                let delay: f32 = rng.gen_range(obstacle.delay_start..obstacle.delay_end);
                obstacle.delay = Timer::from_seconds(delay, false);
            } else if transform.translation.x < 0. {
                can_spawn.0 = true;
            } else if transform.translation.x < WINDOW_RIGHT {
                can_spawn.0 = false;
            }
        } else if can_spawn.0 && obstacle.delay.tick(time.delta()).just_finished() {
            obstacle.moving = true;
            can_spawn.0 = false;
            let exponent: u16 = (score_stopwatch.0.elapsed_secs() / 0.9) as u16;
            obstacle.velocity = INITIAL_VELOCITY_OBSTACLE * 1.01_f32.powf(exponent as f32);
        }
    }
}

fn keyboard_input(keyboard_input: Res<Input<KeyCode>>, mut sprite_jump_state: Query<&mut Jumping>) {
    for mut jump_state in &mut sprite_jump_state {
        if let Jumping::None = *jump_state {
            if keyboard_input.just_pressed(KeyCode::Space) {
                *jump_state = Jumping::Jump(INITIAL_VELOCITY_PLAYER);
            }
        }
    }
}
