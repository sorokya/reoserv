use eolib::protocol::map::MapTileSpec;

pub fn get_board_tile_spec(board_id: i32) -> Option<MapTileSpec> {
    match board_id {
        1 => Some(MapTileSpec::Board1),
        2 => Some(MapTileSpec::Board2),
        3 => Some(MapTileSpec::Board3),
        4 => Some(MapTileSpec::Board4),
        5 => Some(MapTileSpec::Board5),
        6 => Some(MapTileSpec::Board6),
        7 => Some(MapTileSpec::Board7),
        8 => Some(MapTileSpec::Board8),
        _ => {
            warn!("{} is not a valid board id", board_id);
            None
        }
    }
}
