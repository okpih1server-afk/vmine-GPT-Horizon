fn tick(game: &mut crate::game::Game<'_>) {
    if let Err(error) = game.veinminer_tick() {
        eprintln!("Veinminer delayed mining failed: {error}");
    }
}
