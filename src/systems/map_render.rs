use crate::prelude::*;

#[system]
#[read_component(Player)]
#[read_component(FieldOfView)]
#[read_component(BigFieldOfView)]
#[read_component(Light)]
pub fn map_render(ecs: &SubWorld, #[resource] map: &Map, #[resource] camera: &Camera) {
    let mut fov = <&FieldOfView>::query().filter(component::<Player>());
    let mut fov_light = <&FieldOfView>::query().filter(component::<Light>());
    let mut big_fov = <&BigFieldOfView>::query().filter(component::<Player>());
    let mut draw_batch = DrawBatch::new();
    draw_batch.target(0);
    let player_fov = fov.iter(ecs).next().unwrap();
    let player_big_fov = big_fov.iter(ecs).next().unwrap();
    for y in camera.top_y..=camera.bottom_y {
        for x in camera.left_x..camera.right_x {
            let pt = Point::new(x, y);
            let offset = Point::new(camera.left_x, camera.top_y);
            let idx = map_idx(x, y);
            if map.in_bound(pt) && player_fov.visible_tiles.contains(&pt) | map.revealed_tiles[idx]
            {
                let tint = if fov_light.iter(ecs).any(|light_fov| {
                    light_fov.visible_tiles.contains(&pt)
                        && player_big_fov.visible_tiles.contains(&pt)
                }) {
                    LIGHTYELLOW
                } else if fov_light.iter(ecs).any(|light_fov| {
                    light_fov.visible_tiles.contains(&pt) && player_fov.visible_tiles.contains(&pt)
                }) {
                    LIGHTYELLOW
                } else if player_fov.visible_tiles.contains(&pt) {
                    WHITE
                } else {
                    DARKGRAY
                };
                let glyph = match map.tiles[idx] {
                    TileType::Floor => to_cp437('.'),
                    TileType::Wall => to_cp437('#'),
                };
                draw_batch.set(pt - offset, ColorPair::new(tint, BLACK), glyph);
            }
        }
    }
    draw_batch.submit(0).expect("Batch Error")
}
