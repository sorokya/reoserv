use eo::{
    protocol::Coords,
    pubs::{EmfSpecRow, EmfTileSpec, EmfWarpRow},
};

pub fn is_tile_walkable(coords: Coords, tile_rows: &[EmfSpecRow]) -> bool {
    if let Some(tile_row) = tile_rows.iter().find(|tile_row| tile_row.y == coords.y) {
        if let Some(tile) = tile_row.tiles.iter().find(|tile| tile.x == coords.x) {
            !matches!(
                tile.spec,
                EmfTileSpec::Wall
                    | EmfTileSpec::ChairDown
                    | EmfTileSpec::ChairLeft
                    | EmfTileSpec::ChairRight
                    | EmfTileSpec::ChairUp
                    | EmfTileSpec::ChairDownRight
                    | EmfTileSpec::ChairUpLeft
                    | EmfTileSpec::ChairAll
                    | EmfTileSpec::Chest
                    | EmfTileSpec::BankVault
                    | EmfTileSpec::MapEdge
                    | EmfTileSpec::Board1
                    | EmfTileSpec::Board2
                    | EmfTileSpec::Board3
                    | EmfTileSpec::Board4
                    | EmfTileSpec::Board5
                    | EmfTileSpec::Board6
                    | EmfTileSpec::Board7
                    | EmfTileSpec::Board8
                    | EmfTileSpec::Jukebox
            )
        } else {
            true
        }
    } else {
        true
    }
}

pub fn is_tile_walkable_for_npc(
    coords: Coords,
    tile_rows: &[EmfSpecRow],
    warp_rows: &[EmfWarpRow],
) -> bool {
    if let Some(warp_row) = warp_rows.iter().find(|warp_row| warp_row.y == coords.y) {
        if let Some(_warp) = warp_row.tiles.iter().find(|warp| warp.x == coords.x) {
            return false;
        }
    }

    if let Some(tile_row) = tile_rows.iter().find(|tile_row| tile_row.y == coords.y) {
        if let Some(tile) = tile_row.tiles.iter().find(|tile| tile.x == coords.x) {
            !matches!(
                tile.spec,
                EmfTileSpec::NPCBoundary
                    | EmfTileSpec::Wall
                    | EmfTileSpec::ChairDown
                    | EmfTileSpec::ChairLeft
                    | EmfTileSpec::ChairRight
                    | EmfTileSpec::ChairUp
                    | EmfTileSpec::ChairDownRight
                    | EmfTileSpec::ChairUpLeft
                    | EmfTileSpec::ChairAll
                    | EmfTileSpec::Chest
                    | EmfTileSpec::BankVault
                    | EmfTileSpec::MapEdge
                    | EmfTileSpec::Board1
                    | EmfTileSpec::Board2
                    | EmfTileSpec::Board3
                    | EmfTileSpec::Board4
                    | EmfTileSpec::Board5
                    | EmfTileSpec::Board6
                    | EmfTileSpec::Board7
                    | EmfTileSpec::Board8
                    | EmfTileSpec::Jukebox
            )
        } else {
            true
        }
    } else {
        true
    }
}
