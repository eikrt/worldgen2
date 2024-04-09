use crate::bitmap::*;
use crate::util::{MainMsg, RenderMsg};
use crate::worldgen::{Chunk, Coords, Faction, CHUNK_SIZE, TILE_SIZE};
use lazy_static::lazy_static;
use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use std::collections::HashMap;
use std::time::Duration;
lazy_static! {
    pub static ref WINDOW_WIDTH: u32 = 1240;
    pub static ref WINDOW_HEIGHT: u32 = 760;
    pub static ref DEFAULT_ZOOM: i32 = 1;
    pub static ref CAMERA_STEP: i32 = 32;
}
#[derive(Clone)]
pub struct Camera {
    pub coords: Coords,
    pub ccoords: Coords,
    pub render_distance_w: i32,
    pub render_distance_h: i32,
    pub zoom: i32,
}
impl Camera {
    pub fn new() -> Camera {
        Camera {
            coords: Coords::new(),
            ccoords: Coords::new(),
            render_distance_w: *WINDOW_WIDTH as i32,
            render_distance_h: *WINDOW_HEIGHT as i32,
            zoom: *DEFAULT_ZOOM,
        }
    }
    pub fn tick(&mut self) {
        self.ccoords.x = self.coords.x / *CHUNK_SIZE as i32;
        self.ccoords.y = self.coords.y / *CHUNK_SIZE as i32;
    }
}
pub fn render_server(
    sx: &crossbeam::channel::Sender<MainMsg>,
    rx: &crossbeam::channel::Receiver<Vec<RenderMsg>>,
) {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("Baltia", *WINDOW_WIDTH, *WINDOW_HEIGHT)
        .position_centered()
        .fullscreen_desktop()
        .build()
        .unwrap();
    let mut camera = Camera::new();
    let ttf_context = sdl2::ttf::init().unwrap();
    let font_path = "fonts/VastShadow-Regular.ttf";
    let _font = ttf_context.load_font(font_path, 48).unwrap();
    let mut canvas = window.into_canvas().build().unwrap();
    canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
    let _texture_creator = canvas.texture_creator();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut factions = false;
    let mut news = false;
    let mut trigger_refresh = false;
    'main: loop {
        camera.tick();
        if let Ok(r) = rx.try_recv() {
            for message in r {
                let chunk = message.chunk;
                if chunk.coords.x as i32 * camera.zoom < (camera.ccoords.x) * camera.zoom
                    || chunk.coords.y as i32 * camera.zoom < (camera.ccoords.y) * camera.zoom
                    || chunk.coords.x as i32 * camera.zoom
                        > (camera.ccoords.x + *WINDOW_WIDTH as i32 * *CHUNK_SIZE as i32)
                            * camera.zoom
                    || chunk.coords.y as i32 * camera.zoom
                        > (camera.ccoords.y + *WINDOW_HEIGHT as i32 * *CHUNK_SIZE as i32)
                            * camera.zoom
                {
                    continue;
                }
                for event in event_pump.poll_iter() {
                    match event {
                        Event::Quit { .. }
                        | Event::KeyDown {
                            keycode: Some(Keycode::Escape),
                            ..
                        } => {
                            break 'main;
                        }
                        Event::KeyDown {
                            keycode: Some(Keycode::Plus),
                            ..
                        } => {
                            camera.zoom += 1;
                        }

                        Event::KeyDown {
                            keycode: Some(Keycode::Minus),
                            ..
                        } => {
                            camera.zoom -= 1;
                        }
                        Event::KeyDown {
                            keycode: Some(Keycode::Left),
                            ..
                        } => {
                            camera.coords.x += *CAMERA_STEP;
                            trigger_refresh = true;
                        }
                        Event::KeyDown {
                            keycode: Some(Keycode::Right),
                            ..
                        } => {
                            camera.coords.x -= *CAMERA_STEP;
                            trigger_refresh = true;
                        }
                        Event::KeyDown {
                            keycode: Some(Keycode::Up),
                            ..
                        } => {
                            camera.coords.y += *CAMERA_STEP;
                            trigger_refresh = true;
                        }
                        Event::KeyDown {
                            keycode: Some(Keycode::Down),
                            ..
                        } => {
                            camera.coords.y -= *CAMERA_STEP;
                            trigger_refresh = true;
                        }
                        Event::KeyDown {
                            keycode: Some(Keycode::F),
                            ..
                        } => {
                            factions = !factions;
                        }
                        Event::KeyDown {
                            keycode: Some(Keycode::N),
                            ..
                        } => {
                            news = !news;
                            canvas.set_draw_color(Color::RGB(0, 0, 0));
                            canvas.clear();
                        }
                        Event::Window { win_event, .. } => match win_event {
                            WindowEvent::Resized(width, height) => {
                                canvas
                                    .window_mut()
                                    .set_size(width as u32, height as u32)
                                    .unwrap();
                                camera.render_distance_w = width;
                                camera.render_distance_h = height;
                                canvas.present();
                            }
                            _ => {}
                        },

                        _ => {}
                    }
                }
                if trigger_refresh {
                    canvas.set_draw_color(Color::RGB(0, 0, 0));
                    canvas.clear();
                    trigger_refresh = false;
                }
                if news {
                    let mut row = 0;
                    let mut index = 0;
                    for (i, n) in message.news.newscast.iter().enumerate() {
                        let mut text = n;
                        let char_span = 8;
                        let row_span = 14;

                        for c in text.chars() {
                            let v = LETTERS.get(&c).unwrap().clone();

                            if c == '\n' {
                                row += 1;
                                index = -row;
                                continue;
                            }
                            for (k2, v2) in v.map {
                                canvas.set_draw_color(Color::RGB(255, 255, 255));
                                match v2 {
                                    '#' => {
                                        let _ = canvas.fill_rect(Rect::new(
                                            k2.0 * *TILE_SIZE as i32 * camera.zoom
                                                + camera.coords.x
                                                + index * char_span,
                                            k2.1 * *TILE_SIZE as i32 * camera.zoom
                                                + camera.coords.y
                                                + row * row_span
                                                + 16
                                                + i as i32 * row_span,
                                            *TILE_SIZE * camera.zoom as u32,
                                            *TILE_SIZE * camera.zoom as u32,
                                        ));
                                    }
                                    _ => {}
                                }
                            }
                            index += 1;
                        }
                        row += 1;
                        index = 0;
                    }
                    continue;
                }
                for m in chunk.tiles {
                    let mut color = (
                        (255.0 - (1.0 * m.height as f32 / 0.0) * 255.0) as u8,
                        (255.0 - (1.0 * m.height as f32 / 10.0) * 255.0) as u8,
                        (255.0 - (1.0 * m.height as f32 / 0.0) * 255.0) as u8,
                    );
                    if m.height < 0 {
                        color.0 = 0;
                        color.1 = 0;
                        color.2 = 255;
                    }
                    canvas.set_draw_color(Color::RGB(color.0, color.1, color.2));
                    let _ = canvas.fill_rect(Rect::new(
                        m.coords.x * *TILE_SIZE as i32 * camera.zoom + camera.coords.x,
                        m.coords.y * *TILE_SIZE as i32 * camera.zoom + camera.coords.y,
                        *TILE_SIZE * camera.zoom as u32,
                        *TILE_SIZE * camera.zoom as u32,
                    ));
                }
                for m in &chunk.entities {
                    let mut color = ((0) as u8, 255 as u8, 0);
                    color.0 = 255;
                    color.1 = 0;
                    color.2 = 0;
                    canvas.set_draw_color(Color::RGB(color.0, color.1, color.2));
                    let _ = canvas.fill_rect(Rect::new(
                        m.coords.x * *TILE_SIZE as i32 * camera.zoom + camera.coords.x,
                        m.coords.y * *TILE_SIZE as i32 * camera.zoom + camera.coords.y,
                        *TILE_SIZE * camera.zoom as u32,
                        *TILE_SIZE * camera.zoom as u32,
                    ));
                }
                if factions {
                    let counts: HashMap<Faction, usize> =
                        chunk
                            .entities
                            .clone()
                            .iter()
                            .fold(HashMap::new(), |mut acc, entity| {
                                *acc.entry(entity.clone().alignment.faction).or_insert(0) += 1;
                                acc
                            });
                    let max_value = counts
                        .iter()
                        .max_by_key(|&(_, v)| v)
                        .map(|(k, _)| k)
                        .unwrap_or(&Faction::Empty);

                    match max_value {
                        &Faction::Empty => {
                            canvas.set_draw_color(Color::RGBA(0, 0, 0, 100));
                        }
                        &Faction::Hiisi => {
                            canvas.set_draw_color(Color::RGBA(255, 255, 255, 100));
                        }
                        &Faction::Virumaa => {
                            canvas.set_draw_color(Color::RGBA(0, 0, 255, 100));
                        }
                        &Faction::Kalevala => {
                            canvas.set_draw_color(Color::RGBA(255, 255, 0, 100));
                        }
                        &Faction::Pohjola => {
                            canvas.set_draw_color(Color::RGBA(0, 0, 255, 100));
                        }
                        &Faction::Tapiola => {
                            canvas.set_draw_color(Color::RGBA(0, 255, 0, 100));
                        }
                        &Faction::Novgorod => {
                            canvas.set_draw_color(Color::RGBA(255, 0, 0, 100));
                        }
                    };
                    let _ = canvas.fill_rect(Rect::new(
                        chunk.coords.x * *CHUNK_SIZE as i32 * camera.zoom + camera.coords.x,
                        chunk.coords.y * *CHUNK_SIZE as i32 * camera.zoom + camera.coords.y,
                        *CHUNK_SIZE * *TILE_SIZE * camera.zoom as u32,
                        *CHUNK_SIZE * *TILE_SIZE * camera.zoom as u32,
                    ));
                }
            }
            canvas.present();
            let _ = sx.send(MainMsg::from(camera.clone(), true));
            ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
        }
    }
}
