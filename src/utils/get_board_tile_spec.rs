use eo::{data::EOShort, pubs::EmfTileSpec};

pub fn get_board_tile_spec(board_id: EOShort) -> Option<EmfTileSpec> {
    match board_id {
        0 => Some(EmfTileSpec::Board1),
        1 => Some(EmfTileSpec::Board2),
        2 => Some(EmfTileSpec::Board3),
        3 => Some(EmfTileSpec::Board4),
        4 => Some(EmfTileSpec::Board5),
        5 => Some(EmfTileSpec::Board6),
        6 => Some(EmfTileSpec::Board7),
        7 => Some(EmfTileSpec::Board8),
        _ => {
            warn!("{} is not a valid board id", board_id);
            None
        }
    }
}
