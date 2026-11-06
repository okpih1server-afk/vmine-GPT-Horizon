fn redirect(
    game: &mut crate::game::Game<'_>,
    stack: &horizon_client::items::ItemStack,
    position: [f64; 3],
    velocity: [f64; 3],
    pickup_delay: u32,
) -> core::ops::ControlFlow<()> {
    game.veinminer_redirect_drop(stack, position, velocity, pickup_delay)
}
