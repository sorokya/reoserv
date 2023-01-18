use eo::{
    protocol::Coords,
    pubs::{EmfWarp, EmfWarpRow},
};

pub fn get_warp_at(coords: Coords, warp_rows: &[EmfWarpRow]) -> Option<EmfWarp> {
    if let Some(warp_row) = warp_rows.iter().find(|warp_row| warp_row.y == coords.y) {
        warp_row
            .tiles
            .iter()
            .find(|tile| tile.x == coords.x)
            .map(|tile| tile.warp.to_owned())
    } else {
        None
    }
}
