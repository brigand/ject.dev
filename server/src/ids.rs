use nanoid::nanoid;

/// To test different nanoid lengths: https://zelark.github.io/nano-id-cc/
/// Alphabet: 123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz
pub static BASE58_ALPHA: &[char] = &[
    '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'J', 'K',
    'L', 'M', 'N', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', 'a', 'b', 'c', 'd', 'e',
    'f', 'g', 'h', 'i', 'j', 'k', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y',
    'z',
];

/// Generate a session id. This should be somewhat unguessable, but this might be a bit excessive.
/// At 10 million per hour, it would take forever ot have a collision.
pub fn make_session_id() -> String {
    nanoid!(18, BASE58_ALPHA)
}

/// Generate a save_id.
pub fn make_save_id() -> String {
    // Should be long enough for 1k/hr over thousands of years.
    nanoid!(13, BASE58_ALPHA)
}
