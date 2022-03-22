use eo::{
    data::{
        map::{Warp, WarpRow},
        EOShort,
    },
    world::Coords,
};

pub fn get_warp_at(coords: Coords, warp_rows: &[WarpRow]) -> Option<Warp> {
    if let Some(warp_row) = warp_rows
        .iter()
        .find(|warp_row| warp_row.y as EOShort == coords.y)
    {
        warp_row
            .tiles
            .iter()
            .find(|warp| warp.x as EOShort == coords.x)
            .map(|warp| warp.to_owned())
    } else {
        None
    }
}
