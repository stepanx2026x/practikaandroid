//! Сакура и ёкаи — аниме-кликер для компьютера.
#![allow(clippy::too_many_arguments)]
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod art;
mod data;
mod fmt;
mod game;
mod gfx;
mod input;
mod preview;
mod save;
mod scene;
mod ui;

use macroquad::prelude::*;

use game::Game;
use ui::{Modal, Panel};

fn window_conf() -> Conf {
    Conf {
        window_title: "Сакура и ёкаи".to_string(),
        window_width: 1280,
        window_height: 800,
        high_dpi: true,
        sample_count: 4,
        window_resizable: true,
        icon: Some(macroquad::miniquad::conf::Icon {
            small: art::icon_pixels::<1024>(16),
            medium: art::icon_pixels::<4096>(32),
            big: art::icon_pixels::<16384>(64),
        }),
        ..Default::default()
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 2 && args[1] == "--preview" {
        preview::run(&args[2]);
        return;
    }
    macroquad::Window::from_config(window_conf(), run());
}

async fn run() {
    let mut gfx = gfx::Gfx::new();
    gfx.begin_frame();
    let mut art = art::Art::new(gfx.pixel_scale());
    let mut game = Game::new();
    game.seed(save::now_secs() ^ 0x5EED_CAFE);
    let away = save::load(&mut game);
    let mut scene = scene::Scene::new(&game);
    let mut panel = Panel::new();
    let mut input = input::Input::new();

    match away {
        None => panel.modal = Some(Modal::Welcome),
        Some(secs) if secs > 60.0 => {
            let gold = game.offline_gold(secs);
            if gold >= 1.0 {
                game.earn(gold);
                panel.modal = Some(Modal::Offline {
                    gold,
                    secs: secs.min(game::OFFLINE_LIMIT_SEC),
                });
            }
        }
        _ => {}
    }

    prevent_quit();
    let mut save_timer = 0.0f32;
    let mut saved_changes = game.changes;
    let mut scale_wait = 0.0f32;
    let debug = std::env::var("SAKURA_DEBUG").is_ok();
    let (mut fps_frames, mut fps_time) = (0u32, 0.0f32);

    loop {
        let dt = get_frame_time().min(0.1);
        gfx.begin_frame();

        // при изменении размера окна перерисовываем картинки в новом разрешении (с задержкой)
        if (gfx.pixel_scale() - art.scale).abs() / art.scale > 0.03 {
            scale_wait += dt;
            if scale_wait > 0.3 {
                art.set_scale(gfx.pixel_scale());
                scale_wait = 0.0;
            }
        } else {
            scale_wait = 0.0;
        }

        input.update(gfx.mouse());
        let panel_x = gfx.canvas_w - ui::PANEL_W;
        let lay = scene::Layout::new(panel_x, gfx.canvas_h);

        if panel.modal.is_none() {
            let keys = [
                (KeyCode::Key1, KeyCode::Kp1),
                (KeyCode::Key2, KeyCode::Kp2),
                (KeyCode::Key3, KeyCode::Kp3),
                (KeyCode::Key4, KeyCode::Kp4),
            ];
            for (i, (a, b)) in keys.iter().enumerate() {
                if is_key_pressed(*a) || is_key_pressed(*b) {
                    scene.try_skill(&mut game, i);
                }
            }
            if is_key_pressed(KeyCode::Left) && game.stage > 1 {
                game.auto_advance = false;
                game.set_stage(game.stage - 1);
            }
            if is_key_pressed(KeyCode::Right) && game.stage < game.max_stage {
                game.set_stage(game.stage + 1);
            }
            if is_key_pressed(KeyCode::A) {
                game.auto_advance = !game.auto_advance;
                scene.message(if game.auto_advance {
                    "Вперёд: включено".into()
                } else {
                    "Вперёд: выключено".into()
                });
            }
        } else {
            input.blocked = true;
        }

        game.update(dt as f64);
        let events = game.take_events();
        scene.handle(events, &game, &lay);
        scene.update(dt, &game, &lay);

        clear_background(Color::from_rgba(253, 232, 240, 255));
        scene.draw(&mut game, &gfx, &mut art, &input, &lay);
        panel.draw(
            &mut game,
            &gfx,
            &mut art,
            &input,
            &mut scene,
            panel_x,
            gfx.canvas_h,
            dt,
        );
        panel.draw_modal(&mut game, &gfx, &mut art, &mut input, &mut scene);

        if debug {
            fps_frames += 1;
            fps_time += get_frame_time();
            if fps_time > 3.0 {
                eprintln!("fps: {:.1}", fps_frames as f32 / fps_time);
                fps_frames = 0;
                fps_time = 0.0;
            }
        }

        save_timer += dt;
        // сохраняемся раз в 20 секунд или вскоре после покупки
        if save_timer > 20.0 || (game.changes != saved_changes && save_timer > 2.0) {
            saved_changes = game.changes;
            save_timer = 0.0;
            save::save(&game);
        }
        if is_quit_requested() {
            save::save(&game);
            break;
        }
        next_frame().await;
    }
}
