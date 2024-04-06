use crossbeam::channel::unbounded;
use rand::Rng;
use rayon::prelude::*;
use std::thread;
//use std::time::Duration;
use std::io;
use U::plot::plot;
use U::renderer::{render_server, Camera};
use U::util::RenderMsg;
use U::worldgen::worldgen;
fn main() {
    let (tx, rx) = unbounded();
    let (tx2, rx2) = unbounded();
    thread::spawn(move || {
        render_server(&tx2, &rx);
    });
    let mut worlds = vec![];
    let mut rng = rand::thread_rng();
    for _ in 0..1 {
        let seed = rng.gen_range(0..1000);
        worlds.push(worldgen(seed));
    }
    let mut state: Vec<RenderMsg> = vec![];
    let mut step = 0;
    let mut step_increment = 1;
    let mut camera = Camera::new();
    let mut vic_world = 0;
    let mut render = false;
    thread::spawn(move || {
        //plot();
    });

    thread::spawn(move || {
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(e) => e,
            Err(e) => panic!("End..."),
        };
        match input.as_str() {
            "" => "joinks",
            &_ => todo!(),
        }
    });
    loop {
        // stage 1: resolve all tensions in the world
        worlds
            .par_iter_mut()
            .for_each(|c| c.resolve(step_increment));
        //println!("Stage 1 conducted succesfully!");
        // stage 2: resolve all tensions between chunks
        worlds
            .par_iter_mut()
            .for_each(|c| c.resolve_between(step_increment));
        //println!("Stage 2 conducted succesfully!");
        // stage 3: push into rendering buffer
        let mut world = &mut worlds[vic_world];
        for i in 0..world.chunks.len() {
            let chunk = world.fetch_chunk(i as usize);
            state.push(RenderMsg::from(chunk.clone()));
        }
        let _ = tx.send(state.clone());
        //println!("Stage 3 conducted succesfully!");
        // stage 4: render
        //println!("Stage 4 conducted succesfully!");
        step += step_increment;
        if let Ok(x) = rx2.recv() {
            camera = x.camera;
        }
        //println!("Step {} conducted succesfully!", step);
        //::std::thread::sleep(Duration::new(0, 1_000_000_000u32 * 4));
        // cleanup
        state.clear();
    }
}
