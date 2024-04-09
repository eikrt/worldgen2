use crossbeam::channel::unbounded;
use rand::Rng;
use rayon::prelude::*;
use std::io;
use std::thread;
use std::time::Duration;
use U::plot::plot;
use U::renderer::{render_server, Camera};
use U::util::RenderMsg;
use U::worldgen::{worldgen, News, CHUNK_SIZE, WORLD_SIZE};

use lazy_static::lazy_static;
lazy_static! {
    pub static ref PARTITION_SIZE: usize = (*WORLD_SIZE as usize * *WORLD_SIZE as usize) / 16;
}
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
    let mut partition = 0;
    loop {
        if partition >= *PARTITION_SIZE {
            partition = 0;
            // stage 1: resolve all tensions in the world
            worlds
                .par_iter_mut()
                .for_each(|c| c.resolve(step_increment));
            //println!("Stage 1 conducted succesfully!");
            // stage 2: resolve all tensions between chunks
            worlds
                .par_iter_mut()
                .for_each(|c| c.resolve_between(step_increment));
        }
        //println!("Stage 2 conducted succesfully!");
        // stage 3: push into rendering buffer
        let mut world = &mut worlds[vic_world];
        let mut news = News::from(vec!["news ".to_string()]);
        for i in ((world.chunks.len() / *PARTITION_SIZE) * partition)
            ..(((world.chunks.len() / *PARTITION_SIZE) * partition)
                + (world.chunks.len() / *PARTITION_SIZE))
        {
            let chunk = world.fetch_chunk(i as usize);
            if chunk.tiles[0].coords.x < camera.coords.x
                || chunk.tiles[0].coords.y < camera.coords.y
                || chunk.tiles[chunk.tiles.len() - 1].coords.x
                    > -camera.coords.x + camera.render_distance_w + *CHUNK_SIZE as i32
                || chunk.tiles[chunk.tiles.len() - 1].coords.y + *CHUNK_SIZE as i32
                    > -camera.coords.y
                        + camera.render_distance_h
                        + *CHUNK_SIZE as i32
                        + *CHUNK_SIZE as i32
            {
                continue;
            }
            news.newscast.append(&mut chunk.inquire_news().newscast);
            state.push(RenderMsg::from(chunk.clone(), news.clone()));
        }
        let _ = tx.send(state.clone());
        println!("Stage 3 conducted succesfully!");
        // stage 4: render
        println!("Stage 4 conducted succesfully!");
        step += step_increment;
        if let Ok(x) = rx2.recv() {
            camera = x.camera;
        }
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
        // cleanup
        state.clear();
        news.newscast.clear();
        partition += 1;
    }
}
