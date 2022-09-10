use eo::{
    data::{
        map::{TileRow, TileSpec, WarpRow},
    },
    world::TinyCoords,
};

pub fn is_tile_walkable(coords: TinyCoords, tile_rows: &[TileRow]) -> bool {
    if let Some(tile_row) = tile_rows
        .iter()
        .find(|tile_row| tile_row.y == coords.y)
    {
        if let Some(tile) = tile_row
            .tiles
            .iter()
            .find(|tile| tile.x == coords.x)
        {
            !matches!(
                tile.spec,
                TileSpec::Wall
                    | TileSpec::ChairDown
                    | TileSpec::ChairLeft
                    | TileSpec::ChairRight
                    | TileSpec::ChairUp
                    | TileSpec::ChairDownRight
                    | TileSpec::ChairUpLeft
                    | TileSpec::ChairAll
                    | TileSpec::Chest
                    | TileSpec::BankVault
                    | TileSpec::MapEdge
                    | TileSpec::Board1
                    | TileSpec::Board2
                    | TileSpec::Board3
                    | TileSpec::Board4
                    | TileSpec::Board5
                    | TileSpec::Board6
                    | TileSpec::Board7
                    | TileSpec::Board8
                    | TileSpec::Jukebox
            )
        } else {
            true
        }
    } else {
        true
    }
}

pub fn is_tile_walkable_for_npc(coords: TinyCoords, tile_rows: &[TileRow], warp_rows: &[WarpRow]) -> bool {
    if let Some(warp_row) = warp_rows
        .iter()
        .find(|warp_row| warp_row.y == coords.y) {
            if let Some(warp) = warp_row
                .tiles
                .iter()
                .find(|warp| warp.x == coords.x)
            {
                return false;
            }
        }

    if let Some(tile_row) = tile_rows
        .iter()
        .find(|tile_row| tile_row.y == coords.y)
    {
        if let Some(tile) = tile_row
            .tiles
            .iter()
            .find(|tile| tile.x == coords.x)
        {
            !matches!(
                tile.spec,
                TileSpec::NPCBoundary
                    | TileSpec::Wall
                    | TileSpec::ChairDown
                    | TileSpec::ChairLeft
                    | TileSpec::ChairRight
                    | TileSpec::ChairUp
                    | TileSpec::ChairDownRight
                    | TileSpec::ChairUpLeft
                    | TileSpec::ChairAll
                    | TileSpec::Chest
                    | TileSpec::BankVault
                    | TileSpec::MapEdge
                    | TileSpec::Board1
                    | TileSpec::Board2
                    | TileSpec::Board3
                    | TileSpec::Board4
                    | TileSpec::Board5
                    | TileSpec::Board6
                    | TileSpec::Board7
                    | TileSpec::Board8
                    | TileSpec::Jukebox
            )
        } else {
            true
        }
    } else {
        true
    }
}
