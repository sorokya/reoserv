const MAX_TIMESTAMP: i32 = 8640000;

pub fn timestamp_diff(a: i32, b: i32) -> i32 {
    match (a, b) {
        (-1, _) => b,
        (_, -1) => a,
        _ => {
            if b > a {
                a - b + MAX_TIMESTAMP
            } else {
                a - b
            }
        }
    }
}
