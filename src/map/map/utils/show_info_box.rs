use eolib::{
    data::EoWriter,
    protocol::net::{PacketAction, PacketFamily},
};

use super::super::Map;

// Copied from EOSERV
const SIZES: [i32; 256] = [
    3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, // NUL - SI
    3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, // DLE - US
    3, 3, 5, 7, 6, 8, 6, 2, 3, 3, 4, 6, 3, 3, 3, 5, // ' ' - '/'
    6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 3, 3, 6, 6, 6, 6, // '0' - '?'
    11, 7, 7, 7, 8, 7, 6, 8, 8, 3, 5, 7, 6, 9, 8, 8, // '@' - 'O'
    7, 8, 8, 7, 7, 8, 7, 11, 7, 7, 7, 3, 5, 3, 6, 6, // 'P' - '_'
    3, 6, 6, 6, 6, 6, 3, 6, 6, 2, 2, 6, 2, 8, 6, 6, // '`' - 'o'
    6, 6, 3, 5, 3, 6, 6, 8, 5, 5, 5, 4, 2, 4, 7, 3, // 'p' - DEL
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    3, 3, 6, 6, 6, 6, 2, 6, 3, 9, 4, 6, 6, 3, 8, 6, 4, 6, 3, 3, 3, 6, 6, 3, 3, 3, 4, 6, 8, 8, 8, 6,
    7, 7, 7, 7, 7, 7, 10, 7, 7, 7, 7, 7, 3, 3, 3, 3, 8, 8, 8, 8, 8, 8, 8, 6, 8, 8, 8, 8, 8, 7, 7,
    6, 6, 6, 6, 6, 6, 6, 10, 6, 6, 6, 6, 6, 2, 4, 4, 4, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 5,
    6, 5,
];

fn text_width(string: &str) -> i32 {
    let mut length = 0;

    for c in string.chars() {
        length += SIZES[c as usize & 0xFF];
    }

    length
}

fn text_cap(string: &str, width: i32, ellipses: &str) -> String {
    let mut length = 0;

    for (i, c) in string.chars().enumerate() {
        length += SIZES[c as usize & 0xFF];

        if length > width {
            let ellipses_length = text_width(ellipses);

            let mut i = i;
            while length > (width + ellipses_length) && i > 0 {
                length -= SIZES[string.chars().nth(i).unwrap() as usize & 0xFF];
                i -= 1;
            }

            return format!("{}{}", &string[..i], ellipses);
        }
    }

    string.to_string()
}

impl Map {
    // Adapted from code Cirras shared:
    // https://discord.com/channels/723989119503696013/787685796055482368/1035174564264681552
    pub fn show_info_box(&mut self, player_id: i32, title: &str, lines: Vec<&str>) {
        let player = match self.characters.get(&player_id) {
            Some(character) => match character.player.as_ref() {
                Some(player) => player,
                None => return,
            },
            None => return,
        };

        const INFOBOX_WIDTH: usize = 197;
        const SPACE_WIDTH: usize = 3;

        let mut content = String::new();
        let mut content_line = String::new();
        let mut content_line_width: usize;

        let add_current_content_line =
            |content: &mut String, content_line: &mut String, content_line_width: usize| {
                if content_line.is_empty() {
                    return;
                }

                content.push_str(content_line);
                let mut spaces = (INFOBOX_WIDTH - content_line_width) / SPACE_WIDTH;

                if content_line.starts_with(' ') {
                    spaces += 1;
                }

                content.extend(std::iter::repeat(' ').take(spaces));
                content_line.clear();
            };

        content.reserve(lines.len() * 2 * INFOBOX_WIDTH);
        content_line.reserve(INFOBOX_WIDTH);

        for line in lines.iter() {
            let mut chunks = Vec::new();
            let mut chunk = String::new();
            let mut leading_whitespace = false;

            for c in line.chars() {
                if c == ' ' {
                    if chunks.is_empty() {
                        leading_whitespace = true;
                    }

                    if leading_whitespace {
                        chunk.push(c);
                    } else if !chunk.is_empty() {
                        chunk.push(c);
                        chunks.push(chunk.clone());
                        chunk.clear();
                    }
                    continue;
                }

                if leading_whitespace {
                    leading_whitespace = false;
                    chunks.push(chunk.clone());
                    chunk.clear();
                }

                chunk.push(c);
            }

            if chunks.is_empty() && chunk.is_empty() {
                chunk.push(' ');
            }

            if !chunk.is_empty() {
                chunks.push(chunk);
            }

            content_line.clear();
            content_line_width = 0;

            for chunk in chunks {
                let chunk_width = text_width(&chunk);

                if content_line_width + chunk_width as usize > INFOBOX_WIDTH {
                    add_current_content_line(&mut content, &mut content_line, content_line_width);

                    if chunk_width >= INFOBOX_WIDTH as i32 {
                        content.push_str(&text_cap(&chunk, INFOBOX_WIDTH as i32, ".."));
                        continue;
                    }
                }

                content_line.push_str(&chunk);
                content_line_width += chunk_width as usize;
            }

            add_current_content_line(&mut content, &mut content_line, content_line_width);
        }

        let mut writer = EoWriter::new();
        writer.add_string(title);
        writer.add_byte(0xff);
        writer.add_string(&content);

        let buf = writer.to_byte_array();

        player.send_buf(PacketAction::Accept, PacketFamily::Message, buf);
    }
}
