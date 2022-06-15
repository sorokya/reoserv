use eo::{
    data::{
        map::{Warp, WarpRow},
    },
    world::TinyCoords,
};

pub fn get_warp_at(coords: TinyCoords, warp_rows: &[WarpRow]) -> Option<Warp> {
    if let Some(warp_row) = warp_rows
        .iter()
        .find(|warp_row| warp_row.y == coords.y)
    {
        warp_row
            .tiles
            .iter()
            .find(|warp| warp.x == coords.x)
            .map(|warp| warp.to_owned())
    } else {
        None
    }
}
