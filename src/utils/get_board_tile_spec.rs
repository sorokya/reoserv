use eo::{data::i32, pubs::EmfTileSpec};

pub fn get_board_tile_spec(board_id: i32) -> Option<EmfTileSpec> {
    match board_id {
        1 => Some(EmfTileSpec::Board1),
        2 => Some(EmfTileSpec::Board2),
        3 => Some(EmfTileSpec::Board3),
        4 => Some(EmfTileSpec::Board4),
        5 => Some(EmfTileSpec::Board5),
        6 => Some(EmfTileSpec::Board6),
        7 => Some(EmfTileSpec::Board7),
        8 => Some(EmfTileSpec::Board8),
        _ => {
            warn!("{} is not a valid board id", board_id);
            None
        }
    }
}
