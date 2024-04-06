use crate::renderer::Camera;
use crate::worldgen::Chunk;

#[derive(Clone)]
pub struct RenderMsg {
    pub chunk: Chunk,
}
impl RenderMsg {
    pub fn from(chunk: Chunk) -> RenderMsg {
        RenderMsg { chunk: chunk }
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
