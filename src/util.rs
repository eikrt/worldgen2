use crate::renderer::Camera;
use crate::worldgen::{Chunk, News};

#[derive(Clone)]
pub struct RenderMsg {
    pub chunk: Chunk,
    pub news: News,
}
impl RenderMsg {
    pub fn from(chunk: Chunk, news: News) -> RenderMsg {
        RenderMsg {
            chunk: chunk,
            news: news,
        }
    }
}
#[derive(Clone)]
pub struct MainMsg {
    pub camera: Camera,
    pub ok: bool,
}
impl MainMsg {
    pub fn from(camera: Camera, ok: bool) -> MainMsg {
        MainMsg {
            camera: camera,
            ok: ok,
        }
    }
}
