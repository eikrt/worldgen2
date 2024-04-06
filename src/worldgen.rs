use crate::math::dist;
use lazy_static::lazy_static;
use noise::{NoiseFn, Perlin};
use rand::Rng;
use rayon::prelude::*;
lazy_static! {
    pub static ref WORLD_SIZE: u32 = 8;
    pub static ref CHUNK_SIZE: u32 = 128;
    pub static ref TILE_SIZE: u32 = 1;
    pub static ref NOISE_SCALE: f64 = 64.0;
    pub static ref VICINITY_DIST: i32 = 4;
}
#[derive(Clone)]
pub struct Stats {
    health: i8,
    hunger: u8,
    strength: u8,
    intelligence: u8,
    agility: u8,
}
#[derive(Clone)]
pub enum Faction {
    Empty,
}
#[derive(Clone)]
pub struct Personality {
    aggression: u8,
}
impl Personality {
    pub fn new() -> Personality {
        Personality { aggression: 0 }
    }
    pub fn gen() -> Personality {
        let mut rng = rand::thread_rng();
        Personality {
            aggression: rng.gen_range(0..100),
        }
    }
}
#[derive(Clone)]
pub struct Alignment {
    faction: Faction,
    personality: Personality,
}
impl Alignment {
    pub fn new() -> Alignment {
        Alignment {
            faction: Faction::Empty,
            personality: Personality::gen(),
        }
    }
    pub fn from(faction: Faction) -> Alignment {
        Alignment {
            faction: faction,
            personality: Personality::gen(),
        }
    }
}
impl Stats {
    pub fn new() -> Stats {
        Stats {
            health: 100,
            hunger: 100,
            strength: 10,
            intelligence: 10,
            agility: 10,
        }
    }
    pub fn gen() -> Stats {
        let mut rng = rand::thread_rng();
        Stats {
            health: 100,
            hunger: 100,
            strength: rng.gen_range(0..10),
            intelligence: rng.gen_range(0..10),
            agility: rng.gen_range(0..10),
        }
    }
}
#[derive(Clone, PartialEq)]
pub enum Status {
    Talking,
    Fighting,
    Idle,
}
#[derive(Clone)]
pub enum TileType {
    Grass,
}
#[derive(Clone)]
pub enum EntityType {
    Human,
}
#[derive(Clone)]
pub struct Coords {
    pub x: i32,
    pub y: i32,
}

#[derive(Clone)]
pub struct Size {
    pub x: i32,
    pub y: i32,
}
impl Size {
    pub fn from(size: (i32, i32)) -> Size {
        Size {
            x: size.0,
            y: size.1,
        }
    }
}
impl Coords {
    pub fn from(coords: (i32, i32)) -> Coords {
        Coords {
            x: coords.0,
            y: coords.1,
        }
    }
    pub fn new() -> Coords {
        Coords { x: 0, y: 0 }
    }
}
#[derive(Clone)]
pub struct Entity {
    pub coords: Coords,
    pub etype: EntityType,
    pub stats: Stats,
    pub status: Status,
    pub index: usize,
    pub alignment: Alignment,
}
impl Entity {
    pub fn new(index: usize) -> Entity {
        Entity {
            coords: Coords::new(),
            etype: EntityType::Human,
            stats: Stats::new(),
            status: Status::Idle,
            alignment: Alignment::new(),
            index: index,
        }
    }
    pub fn from(
        index: usize,
        coords: Coords,
        etype: EntityType,
        stats: Stats,
        alignment: Alignment,
    ) -> Entity {
        Entity {
            coords: coords,
            etype: etype,
            stats: stats,
            status: Status::Idle,
            index: index,
            alignment: alignment,
        }
    }
    pub fn resolve(&mut self, step_increment: i32) {
        if self.stats.hunger > 0 {
            self.stats.hunger -= 1;
        }
        let mut rng = rand::thread_rng();
        let roll = rng.gen_range(0..10);
        if self.stats.hunger == 0 {
            if self.stats.health >= 0 {
                self.stats.health -= 2;
            } else {
                self.stats.health = 0;
            }
        }
    }
    pub fn resolve_against(&mut self, other: &mut Entity, step_increment: i32) {
        let mut rng = rand::thread_rng();
        let roll = rng.gen_range(0..10);
        if dist(&self.coords, &other.coords) <= *VICINITY_DIST {
            if other.status == Status::Fighting {
                let dmg = other.stats.strength * roll;
                self.stats.health -= dmg as i8;
                if self.alignment.personality.aggression > 25 {
                    self.status = Status::Fighting;
                }
            }
        }
    }
}
#[derive(Clone)]
pub struct Tile {
    pub coords: Coords,
    pub index: usize,
    pub size: Size,
    pub height: i32,
    pub ttype: TileType,
    pub holds: Option<Entity>,
}

impl Tile {
    pub fn from(
        coords: Coords,
        index: usize,
        size: Size,
        height: i32,
        ttype: TileType,
        holds: Option<Entity>,
    ) -> Tile {
        Tile {
            coords,
            index,
            size,
            height,
            ttype,
            holds,
        }
    }
}
#[derive(Clone)]
pub struct Chunk {
    pub tiles: Vec<Tile>,
    pub entities: Vec<Entity>,
    pub coords: Coords,
    pub index: usize,
}

impl Chunk {
    pub fn from(tiles: Vec<Tile>, entities: Vec<Entity>, coords: Coords, index: usize) -> Chunk {
        Chunk {
            tiles,
            entities,
            coords,
            index,
        }
    }
    pub fn new() -> Chunk {
        Chunk {
            tiles: vec![],
            entities: vec![],
            coords: Coords::new(),
            index: 0,
        }
    }
    pub fn resolve(&mut self, step_increment: i32) {
        for i in 0..step_increment {
            for _t in &mut self.tiles {}
            let mut entities_clone = self.entities.clone();
            for clone in &mut entities_clone {
                for entity in &mut self.entities {
                    entity.resolve_against(clone, step_increment);
                }
            }
            for entity in &mut self.entities {
                entity.resolve(step_increment);
            }
            self.entities = self
                .entities
                .iter()
                .filter(|e| e.stats.health > 0)
                .cloned()
                .collect();
        }
    }
    pub fn gen(&mut self, seed: u32) -> Chunk {
        let mut rng = rand::thread_rng();
        let mut tiles: Vec<Tile> = vec![];
        let mut entities: Vec<Entity> = vec![];
        let perlin = Perlin::new(seed);
        let perlin2 = Perlin::new(seed + 1);
        let perlin3 = Perlin::new(seed + 2);
        for c in 0..(*CHUNK_SIZE as i32 * *CHUNK_SIZE as i32) {
            let x = c % (*CHUNK_SIZE as i32) + self.coords.x * *CHUNK_SIZE as i32;
            let y = (c / *CHUNK_SIZE as i32) + self.coords.y * *CHUNK_SIZE as i32;
            let a = 2.0;
            let n1 = perlin.get([
                (x as f64) / *NOISE_SCALE + 0.1,
                (y as f64) / *NOISE_SCALE + 0.1,
            ]) * a;

            let n2 = perlin2.get([
                (x as f64) / *NOISE_SCALE + 0.1,
                (y as f64) / *NOISE_SCALE + 0.1,
            ]) * a
                / 2.0;

            let n3 = perlin3.get([
                (x as f64) / *NOISE_SCALE + 0.1,
                (y as f64) / *NOISE_SCALE + 0.1,
            ]) * a
                / 3.0;
            let height: i32 = (n1 + n2 + n3 + rng.gen_range(-1.0..1.0)) as i32;
            if rng.gen_range(0..32) == 1 {
                entities.push(Entity::from(
                    c as usize,
                    Coords::from((x, y)),
                    EntityType::Human,
                    Stats::gen(),
                    Alignment::from(Faction::Empty),
                ))
            }
            tiles.push(Tile::from(
                Coords::from((x, y)),
                c as usize,
                Size::from((*TILE_SIZE as i32, *TILE_SIZE as i32)),
                height,
                TileType::Grass,
                None,
            ));
        }
        Chunk {
            tiles: tiles,
            entities: entities,
            coords: self.coords.clone(),
            index: self.index,
        }
    }
    pub fn fetch_tile(&self, index: usize) -> &Tile {
        &self.tiles[index]
    }
}
pub struct World {
    pub chunks: Vec<Chunk>,
}
impl World {
    pub fn from(chunks: Vec<Chunk>) -> World {
        World { chunks }
    }
    pub fn fetch_chunk_mut(&mut self, index: usize) -> &mut Chunk {
        &mut self.chunks[index]
    }
    pub fn fetch_chunk(&self, index: usize) -> &Chunk {
        &self.chunks[index]
    }
    pub fn resolve(&mut self, step_increment: i32) {
        self.chunks
            .par_iter_mut()
            .for_each(|c| c.resolve(step_increment));
    }
    pub fn resolve_between(&mut self, step_increment: i32) {}
}
pub fn worldgen(seed: u32) -> World {
    let mut chunks: Vec<Chunk> = vec![];
    for c in 0..((*WORLD_SIZE * *WORLD_SIZE) as i32) {
        let x = c % (*WORLD_SIZE as i32);
        let y = c / *WORLD_SIZE as i32;
        chunks.push(Chunk::from(
            vec![],
            vec![],
            Coords::from((x, y)),
            c as usize,
        ));
    }
    chunks.par_iter_mut().for_each(|c| *c = c.gen(seed));
    let world = World::from(chunks);
    world
}
