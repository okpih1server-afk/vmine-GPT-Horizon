fn tick(game: &mut crate::game::Game<'_>) {
    game.veinminer_key_tick();
    if let Err(error) = game.veinminer_tick() {
        eprintln!("Veinminer delayed mining failed: {error}");
    }
}
