/// Calculate the user's level from the amout of XP they have
pub fn xp_to_lvl(xp: i32) -> i32 {
    if xp < 100 {
        0
    } else {
        (-50 + ((20 * xp + 500) as f32).sqrt().ceil() as i32) / 10
    }
}

/// Caclulate the amount of XP each level costs
pub fn lvl_to_xp(lvl: i32) -> i32 {
    (5 * lvl.pow(2) + 50 * lvl + 100) as i32
}

/// Caclulate the amount of XP needed to reach the next level
pub fn xp_to_next_lvl(lvl: i32, xp: i32) -> i32 {
    (5 * lvl.pow(2) + 5 * lvl + 100) - xp
}
