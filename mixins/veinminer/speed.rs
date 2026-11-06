fn speed(game: &mut crate::game::Game<'_>, pos: [i32; 3], value: f32) -> f32 {
    game.veinminer_speed_multiplier(pos, value)
}
