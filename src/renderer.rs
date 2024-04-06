use crate::util::{MainMsg, RenderMsg};
use crate::worldgen::{Chunk, Coords, CHUNK_SIZE, TILE_SIZE};
use lazy_static::lazy_static;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
lazy_static! {
    pub static ref WINDOW_WIDTH: u32 = 1240;
    pub static ref WINDOW_HEIGHT: u32 = 760;
    pub static ref DEFAULT_ZOOM: i32 = 1;
    pub static ref CAMERA_STEP: i32 = 32;
}
#[derive(Clone)]
pub struct Camera {
    coords: Coords,
    ccoords: Coords,
    zoom: i32,
}
impl Camera {
    pub fn new() -> Camera {
        Camera {
            coords: Coords::new(),
            ccoords: Coords::new(),
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
        .window("U", *WINDOW_WIDTH, *WINDOW_HEIGHT)
        .position_centered()
        .build()
        .unwrap();
    let mut camera = Camera::new();
    let ttf_context = sdl2::ttf::init().unwrap();
    let font_path = "fonts/VastShadow-Regular.ttf";
    let _font = ttf_context.load_font(font_path, 48).unwrap();
    let mut canvas = window.into_canvas().build().unwrap();
    let _texture_creator = canvas.texture_creator();
    let mut event_pump = sdl_context.event_pump().unwrap();
    'main: loop {
        camera.tick();
        if let Ok(r) = rx.try_recv() {
            canvas.set_draw_color(Color::RGB(0, 0, 0));
            canvas.clear();
            for c in r {
                let chunk = c.chunk;
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
                        }
                        Event::KeyDown {
                            keycode: Some(Keycode::Right),
                            ..
                        } => {
                            camera.coords.x -= *CAMERA_STEP;
                        }
                        Event::KeyDown {
                            keycode: Some(Keycode::Up),
                            ..
                        } => {
                            camera.coords.y -= *CAMERA_STEP;
                        }
                        Event::KeyDown {
                            keycode: Some(Keycode::Down),
                            ..
                        } => {
                            camera.coords.y += *CAMERA_STEP;
                        }
                        _ => {}
                    }
                }
                for m in chunk.tiles {
                    let mut color = ((0) as u8, 255 as u8, 0);
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
                for m in chunk.entities {
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
            }
            canvas.present();
        }
        let _ = sx.send(MainMsg::from(camera.clone(), true));
    }
}
