use crate::{prelude::*, map_builder::{prefab::apply_prefab, themes::{DungeonTheme, ForestTheme}}};

mod automata;
mod drunkard;
mod empty;
mod rooms;
mod prefab;
mod themes;

const NULM_ROOMS: usize = 20;
const MIN_ROOM_SIZE: i32 = 3;
const MAX_ROOM_SIZE: i32 = 10;

trait MapArchitect {
    fn new(&mut self, rng: &mut RandomNumberGenerator) -> MapBuilder;
}

pub trait MapTheme: Sync + Send {
    fn tile_to_render(&self, tile_type: TileType) -> FontCharType;
}

pub struct MapBuilder {
    pub map: Map,
    pub rooms: Vec<Rect>,
    pub monster_spawns: Vec<Point>,
    pub player_start: Point,
    pub amulet_start: Point,
    pub theme: Box<dyn MapTheme>,
}

impl MapBuilder {
    pub fn new(rng: &mut RandomNumberGenerator) -> Self {
        let mut architect: Box <dyn MapArchitect> = match rng.range(0, 3) {
            0 => {
                println!("Generate with CellularAutomataArchitect");
                Box::new(automata::CellularAutomataArchitect {})
            },
            1 => {
                println!("Generate with DrunkardWalkArchitect");
                Box::new(drunkard::DrunkardWalkArchitect {})},
            _ => {
                println!("Generate with RoomsArchitect");
                Box::new(rooms::RoomsArchitect {})},
        };

        //let mut architect = empty::EmptyArchitect{};
        let mut mb = architect.new(rng);

        apply_prefab(&mut mb, rng);

        mb.theme = match rng.range(0,2) {
            0 => DungeonTheme::new(),
            _ => ForestTheme::new(),
        };

        println!("{} monster have been generated", mb.monster_spawns.len()); // counting method not accurate with spawn_entity method
        println!(
            "The amulet of Yala has spawn at {},{}",
            mb.amulet_start.x, mb.amulet_start.y
        );
        mb
    }
    fn fill(&mut self, tile: TileType) {
        self.map.tiles.iter_mut().for_each(|t| *t = tile)
    }
    fn find_most_distant(&self) -> Point {
        let dijkstra_map = DijkstraMap::new(
            SCREEN_WIDTH,
            SCREEN_HEIGHT,
            &vec![self.map.point2d_to_index(self.player_start)],
            &self.map,
            1024.0,
        );

        const UNREACHABLE: &f32 = &f32::MAX;
        self.map.index_to_point2d(
            dijkstra_map
                .map
                .iter()
                .enumerate()
                .filter(|(_, dist)| *dist < UNREACHABLE)
                .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
                .unwrap()
                .0,
        )
    }
    fn build_random_rooms(&mut self, rng: &mut RandomNumberGenerator) {
        while self.rooms.len() < NULM_ROOMS {
            let room = Rect::with_size(
                rng.range(1, SCREEN_WIDTH - MAX_ROOM_SIZE - 1),
                rng.range(1, SCREEN_HEIGHT - MAX_ROOM_SIZE - 1),
                rng.range(MIN_ROOM_SIZE, MAX_ROOM_SIZE),
                rng.range(MIN_ROOM_SIZE, MAX_ROOM_SIZE),
            );
            let mut overlap = false;
            for r in self.rooms.iter() {
                if r.intersect(&room) {
                    overlap = true;
                }
            }
            if !overlap {
                room.for_each(|p| {
                    if p.x > 1 && p.x < SCREEN_WIDTH-1 && p.y > 1 && p.y < SCREEN_HEIGHT-1 {
                        let idx = map_idx(p.x, p.y);
                        self.map.tiles[idx] = TileType::Floor;
                    }
                });
                self.rooms.push(room)
            }
        }
    }
    fn apply_vertical_tunnel(&mut self, y1: i32, y2: i32, x: i32) {
        use std::cmp::{max, min};
        for y in min(y1, y2)..=max(y1, y2) {
            if let Some(idx) = self.map.try_idx(Point::new(x, y)) {
                self.map.tiles[idx] = TileType::Floor
            }
        }
    }

    fn apply_horizontal_tunnel(&mut self, x1: i32, x2: i32, y: i32) {
        use std::cmp::{max, min};
        for x in min(x1, x2)..=max(x1, x2) {
            if let Some(idx) = self.map.try_idx(Point::new(x, y)) {
                self.map.tiles[idx] = TileType::Floor
            }
        }
    }
    fn build_corridors(&mut self, rng: &mut RandomNumberGenerator) {
        let mut rooms = self.rooms.clone();
        rooms.sort_by(|a, b| a.center().x.cmp(&b.center().x));

        for (i, room) in rooms.iter().enumerate().skip(1) {
            let prev = rooms[i - 1].center();
            let new = room.center();

            if rng.range(0, 2) == 1 {
                self.apply_horizontal_tunnel(prev.x, new.x, prev.y);
                self.apply_vertical_tunnel(prev.y, new.y, new.x);
            } else {
                self.apply_vertical_tunnel(prev.y, new.y, prev.x);
                self.apply_horizontal_tunnel(prev.x, new.x, new.y);
            }
        }
    }
    fn spawn_monster(&self, start: &Point, rng: &mut RandomNumberGenerator) -> Vec<Point> {
        const NUM_MONSTERS: usize = 50;
        let mut spawnable_tiles: Vec<Point> = self
            .map
            .tiles
            .iter()
            .enumerate()
            .filter(|(idx, t)| {
                **t == TileType::Floor
                    && DistanceAlg::Pythagoras.distance2d(*start, self.map.index_to_point2d(*idx))
                        > 10.0
            })
            .map(|(idx, _)| self.map.index_to_point2d(idx))
            .collect();
        let mut spawns = Vec::new();
        for _ in 0..NUM_MONSTERS {
            let target_index = rng.random_slice_index(&spawnable_tiles).unwrap();
            spawns.push(spawnable_tiles[target_index].clone());
            spawnable_tiles.remove(target_index);
        }
        spawns
    }
}
