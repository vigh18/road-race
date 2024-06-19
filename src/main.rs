use std::f32::consts::PI;

use rand::prelude::*;
use rusty_engine::prelude::*;

#[derive(Resource)]
struct GameState {
    paused: bool,
    health_amount: u8,
    score: u32,
    lost: bool,
    road_speed: f32,
    timer: Timer,
    indestructible_timer: Timer,
    indestructible: bool,
    slowmode_timer: Timer,
    multipier: f32,
}

const PLAYER_SPEED: f32 = 300.0;
const ROAD_SPEED: f32 = 400.0;
const ACCELERATION: f32 = 1.1;
const SLOW_MULTIPIER: f32 = 0.5;

fn main() {
    let mut game = Game::new();

    // game setup goes here
    let player1 = game.add_sprite("player1", SpritePreset::RacingCarBlue);
    player1.scale = 0.8;
    player1.translation.x = -500.0;
    player1.layer = 10.0;
    player1.collision = true;

    let health_message = game.add_text("health_message", "Health: 5");
    health_message.translation = Vec2::new(550.0, 320.0);

    let speed_message = game.add_text("speed_message", format!("Speed: {}", ROAD_SPEED));
    speed_message.translation = Vec2::new(-550.0, 320.0);

    let score_message = game.add_text("score_message", format!("Score: {}", 0));
    score_message.translation = Vec2::new(-550.0, 270.0);

    for i in 0..10 {
        let roadline = game.add_sprite(format!("roadline{}", i), SpritePreset::RacingBarrierWhite);
        roadline.scale = 0.1;
        roadline.translation.x = -600.0 + 150.0 * i as f32;
    }

    let obstacle_presets = vec![
        SpritePreset::RollingBlockNarrow,
        SpritePreset::RollingBlockCorner,
        SpritePreset::RollingBlockSquare,
    ];
    for (i, preset) in obstacle_presets.into_iter().enumerate() {
        let obstacle = game.add_sprite(format!("obstacle{}", i), preset);
        obstacle.rotation = PI / 2.0;
        obstacle.layer = 5.0;
        obstacle.collision = true;
        obstacle.translation.x = thread_rng().gen_range(800.0..1600.0);
        obstacle.translation.y = thread_rng().gen_range(-300.0..300.0);
    }

    for i in 1..5 {
        let score = game.add_sprite(format!("score{}", i), SpritePreset::RollingBallBlue);
        score.layer = 5.0;
        score.collision = true;
        score.translation.x = thread_rng().gen_range(800.0..3200.0);
        score.translation.y = thread_rng().gen_range(-300.0..300.0);
    }

    let buff = game.add_sprite("buff", SpritePreset::RollingBallRed);
    buff.collision = true;
    buff.translation.x = thread_rng().gen_range(800.0..3200.0);
    buff.translation.y = thread_rng().gen_range(-300.0..300.0);

    game.audio_manager
        .play_music(MusicPreset::WhimsicalPopsicle, 0.2);

    game.add_logic(game_logic);
    game.run(GameState {
        paused: false,
        health_amount: 5,
        score: 0,
        lost: false,
        road_speed: ROAD_SPEED,
        timer: Timer::from_seconds(5.0, TimerMode::Repeating),
        indestructible_timer: Timer::from_seconds(0.0, TimerMode::Once),
        indestructible: false,
        slowmode_timer: Timer::from_seconds(0.0, TimerMode::Once),
        multipier: 1.0,
    });
}

fn game_logic(engine: &mut Engine, game_state: &mut GameState) {
    // game logic goes here

    for event in engine.keyboard_events.drain(..) {
        if event.state == ButtonState::Pressed {
            if event.key_code.unwrap() == KeyCode::P {
                game_state.paused = !game_state.paused;
            }
            if event.key_code.unwrap() == KeyCode::N {
                game_state.paused = false;
                game_state.health_amount = 5;
                game_state.score = 0;
                game_state.lost = false;
                game_state.road_speed = ROAD_SPEED;
                game_state.timer = Timer::from_seconds(5.0, TimerMode::Repeating);
                game_state.indestructible_timer = Timer::from_seconds(0.0, TimerMode::Once);
                game_state.indestructible = false;
                game_state.slowmode_timer = Timer::from_seconds(0.0, TimerMode::Once);
                game_state.multipier = 1.0;
            };
        }
    }

    if game_state.lost || game_state.paused {
        return;
    }

    if game_state.timer.tick(engine.delta).just_finished() {
        game_state.road_speed = game_state.road_speed * ACCELERATION;
        let speed_message = engine.texts.get_mut("speed_message").unwrap();
        speed_message.value = format!("Speed: {:.1}", game_state.road_speed);
    }

    if (game_state.indestructible_timer.tick(engine.delta)).just_finished() {
        let player1 = engine.sprites.get_mut("player1").unwrap();
        player1.scale = 0.8;
        game_state.indestructible = false;
    }
    if (game_state.slowmode_timer.tick(engine.delta)).just_finished() {
        game_state.multipier = 1.0;
    }

    let mut direction: f32 = 0.0;
    if engine.keyboard_state.pressed(KeyCode::Up) {
        direction += 1.0;
    }
    if engine.keyboard_state.pressed(KeyCode::Down) {
        direction -= 1.0;
    }

    // move the player
    let player1 = engine.sprites.get_mut("player1").unwrap();
    player1.translation.y += direction * PLAYER_SPEED * engine.delta_f32;
    player1.rotation = direction * 0.2;

    // check boundaries
    if player1.translation.y < -360.0 || player1.translation.y > 360.0 {
        game_state.health_amount = 0;
    }

    // move road objects
    for sprite in engine.sprites.values_mut() {
        if sprite.label.starts_with("roadline") {
            sprite.translation.x -= game_state.road_speed * engine.delta_f32 * game_state.multipier;
            if sprite.translation.x < -675.0 {
                sprite.translation.x += 1500.0;
            }
        }
        if sprite.label.starts_with("buff") || sprite.label.starts_with("score") {
            sprite.translation.x -=
                game_state.road_speed * engine.delta_f32 * 1.25 * game_state.multipier;
            if sprite.translation.x < -800.0 {
                sprite.translation.x = thread_rng().gen_range(800.0..3200.0);
                sprite.translation.y = thread_rng().gen_range(-300.0..300.0);
            }
        }
        if sprite.label.starts_with("obstacle") {
            sprite.translation.x -= game_state.road_speed * engine.delta_f32 * game_state.multipier;
            if sprite.translation.x < -800.0 {
                sprite.translation.x = thread_rng().gen_range(800.0..1600.0);
                sprite.translation.y = thread_rng().gen_range(-300.0..300.0);
            }
        }
    }

    // deal with collisions
    let score_message = engine.texts.get_mut("score_message").unwrap();
    score_message.value = format!("Score: {}", game_state.score);

    let health_message = engine.texts.get_mut("health_message").unwrap();
    health_message.value = format!("Health: {}", game_state.health_amount);

    for event in engine.collision_events.drain(..) {
        // we don't care if obstacles collide with each other or collisions end
        if !event.pair.either_contains("player1") || event.state.is_end() {
            continue;
        }
        if event.pair.either_contains("player1") && event.pair.either_contains("buff") {
            if thread_rng().gen_range(0.0..2.0) > 1.0 {
                game_state.indestructible_timer = Timer::from_seconds(10.0, TimerMode::Once);
                let player1 = engine.sprites.get_mut("player1").unwrap();
                player1.scale = 1.0;
                let buff = engine.sprites.get_mut("buff").unwrap();
                buff.translation.x = thread_rng().gen_range(800.0..3200.0);
                buff.translation.y = thread_rng().gen_range(-300.0..300.0);
                game_state.indestructible = true;
            } else {
                game_state.slowmode_timer = Timer::from_seconds(10.0, TimerMode::Once);
                game_state.multipier = SLOW_MULTIPIER;
            }

            continue;
        }
        if event.pair.either_contains("player1") && event.pair.either_contains("score") {
            let mut label = event.pair.0;
            if event.pair.1.starts_with("score") {
                label = event.pair.1;
            }
            let score = engine.sprites.get_mut(label.as_str()).unwrap();
            score.translation.x = thread_rng().gen_range(800.0..3200.0);
            score.translation.y = thread_rng().gen_range(-300.0..300.0);
            game_state.score += 1;
            continue;
        }
        if !game_state.indestructible && game_state.health_amount > 0 {
            game_state.health_amount -= 1;
            engine.audio_manager.play_sfx(SfxPreset::Impact3, 1.0);
        }
    }

    // finish the game
    if game_state.health_amount == 0 {
        game_state.lost = true;
        let game_over = engine.add_text("game over", "Game Over");
        game_over.font_size = 128.0;
        engine.audio_manager.stop_music();
        engine.audio_manager.play_sfx(SfxPreset::Jingle3, 1.0);
    }
}
