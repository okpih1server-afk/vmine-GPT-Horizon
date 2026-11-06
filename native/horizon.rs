// Native adapter for Horizon 028f60881678a9a0e3e198f693a82858fa475c76.
use super::{Game, Result};
mod engine {
    include!("veinminer_engine.rs");
}
use engine::{Block, Config, Host, Player, Position, Runtime, Tool};
use std::cell::{Cell, RefCell};
use std::collections::VecDeque;

#[derive(Clone)]
struct PendingMine {
    due_tick: i64,
    actor: engine::Actor,
    position: Position,
    expected: Block,
    tool_item: Option<String>,
    preserve_durability: bool,
    drop_target: Option<Position>,
    exhaustion: f32,
}

#[derive(Default)]
pub(super) struct State {
    runtime: Runtime,
    config: Option<Config>,
    pending: VecDeque<PendingMine>,
}

thread_local! {
    static DROP_TARGET: RefCell<Option<[f64; 3]>> = const { RefCell::new(None) };
    static REDIRECTING_DROP: Cell<bool> = const { Cell::new(false) };
}
struct DropTargetGuard(Option<[f64; 3]>);
impl DropTargetGuard {
    fn set(position: Option<Position>) -> Self {
        let target = position.map(|p| [p[0] as f64 + 0.5, p[1] as f64 + 0.5, p[2] as f64 + 0.5]);
        let previous = DROP_TARGET.with(|slot| slot.replace(target));
        Self(previous)
    }
}
impl Drop for DropTargetGuard {
    fn drop(&mut self) { DROP_TARGET.with(|slot| { slot.replace(self.0); }); }
}
struct RedirectGuard;
impl RedirectGuard {
    fn enter() -> Option<Self> {
        REDIRECTING_DROP.with(|flag| if flag.replace(true) { None } else { Some(Self) })
    }
}
impl Drop for RedirectGuard {
    fn drop(&mut self) { REDIRECTING_DROP.with(|flag| flag.set(false)); }
}

thread_local! {
    static STATES: std::cell::RefCell<std::collections::BTreeMap<std::path::PathBuf, State>> = const { std::cell::RefCell::new(std::collections::BTreeMap::new()) };
}
fn with_state<T>(game: &mut Game<'_>, run: impl FnOnce(&mut Game<'_>, &mut State) -> T) -> T {
    struct Lease {
        key: std::path::PathBuf,
        state: State,
    }
    impl Drop for Lease {
        fn drop(&mut self) {
            STATES.with(|states| {
                states
                    .borrow_mut()
                    .insert(self.key.clone(), std::mem::take(&mut self.state));
            });
        }
    }
    let key = game.world_root.clone();
    let state = STATES.with(|states| states.borrow_mut().remove(&key).unwrap_or_default());
    let mut lease = Lease { key, state };
    run(game, &mut lease.state)
}

fn config_path() -> Option<std::path::PathBuf> {
    std::env::var_os("HORIZON_VEINMINER_CONFIG").map(Into::into)
}
fn load_config() -> std::result::Result<Config, String> {
    match config_path() {
        Some(path) => Config::from_json(
            &std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?,
        ),
        None => Ok(Config::default()),
    }
}

struct Native<'g, 'r, 'q> {
    game: &'g mut Game<'r>,
    pending: Option<&'q mut VecDeque<PendingMine>>,
}
impl Host for Native<'_, '_, '_> {
    type Error = Box<dyn std::error::Error>;
    fn player(&mut self) -> Player {
        Player {
            actor: self.game.acting_player_uuid(),
            name: self
                .game
                .lan_acting_name
                .as_deref()
                .unwrap_or(super::chat::PLAYER_NAME)
                .to_owned(),
            tick: self.game.world.game_time,
            survival: self.game.remote.is_none()
                && self.game.player.is_some()
                && self.game.may_build()
                && !self.game.creative,
            sneaking: self
                .game
                .keys
                .contains(&winit::keyboard::KeyCode::ShiftLeft),
            veinmine_pressed: self.game.keys.contains(&winit::keyboard::KeyCode::KeyV),
        }
    }
    fn block(&self, p: Position) -> Option<Block> {
        if p[1] < self.game.world.shape.min_y
            || p[1] >= self.game.world.shape.max_y()
            || !self.game.world.loaded(p[0], p[2])
        {
            return None;
        }
        let state = self.game.world.block(p[0], p[1], p[2]);
        let facts = self.game.res.facts().get(state);
        Some(Block {
            name: facts.name.to_string(),
            state: state.0,
            breakable: facts.hardness >= 0.0 && !self.game.world.is_air(state),
        })
    }
    fn tool(&self) -> Option<Tool> {
        let stack = self.game.held()?;
        let max = self
            .game
            .res
            .items()
            .get(&stack.item)
            .map_or(0, |i| i.max_damage);
        Some(Tool {
            item: stack.item.to_string(),
            remaining: (max > 0).then_some(max.saturating_sub(stack.damage)),
        })
    }
    fn can_harvest(&self, block: &Block) -> bool {
        let facts = self
            .game
            .res
            .facts()
            .get(horizon_world::registry::BlockStateId(block.state));
        !facts.requires_tool
            || self
                .game
                .res
                .items()
                .correct_for_drops(self.game.held(), &block.name)
    }
    fn has_food(&self) -> bool {
        self.game
            .player
            .as_ref()
            .is_some_and(|p| p.health.food.food > 0 || p.health.food.saturation > 0.0)
    }
    fn mine(&mut self, p: Position, preserve: bool) -> Result<bool> {
        self.mine_with_drop_target(p, preserve, None)
    }
    fn mine_with_drop_target(&mut self, p: Position, preserve: bool, drop_target: Option<Position>) -> Result<bool> {
        let _drop_scope = DropTargetGuard::set(drop_target);
        let before = self.block(p);
        let held = preserve.then(|| self.game.inventory[self.game.selected].clone());
        // Temporarily remove pre-existing damage, then restore damage (and a tool
        // destroyed by this one operation) without changing other mining effects.
        if preserve && let Some(stack) = self.game.inventory[self.game.selected].as_mut() {
            stack.damage = 0;
        }
        let result = self.game.break_block_as_player(p);
        if let Some(original) = held {
            match (self.game.inventory[self.game.selected].as_mut(), original) {
                (Some(now), Some(was)) if now.item == was.item => now.damage = was.damage,
                (None, Some(was)) => self.game.inventory[self.game.selected] = Some(was),
                _ => {}
            }
        }
        result?;
        Ok(before.is_some() && self.block(p) != before)
    }
    fn schedule_mine(&mut self, p: Position, expected: &Block, preserve: bool, delay: u32, drop_target: Option<Position>, exhaustion: f32) -> Result<bool> {
        let player = self.player();
        let tool_item = self.tool().map(|tool| tool.item);
        if let Some(queue) = self.pending.as_deref_mut() {
            queue.push_back(PendingMine {
                due_tick: player.tick.saturating_add(i64::from(delay)),
                actor: player.actor,
                position: p,
                expected: expected.clone(),
                tool_item,
                preserve_durability: preserve,
                drop_target,
                exhaustion,
            });
            Ok(true)
        } else {
            let broken = self.mine_with_drop_target(p, preserve, drop_target)?;
            if broken { self.exhaust(exhaustion); }
            Ok(broken)
        }
    }
    fn enchantment_level(&self, id: &str) -> u32 {
        self.game.held().map_or(0, |stack| stack.enchantment_level(id))
    }
    fn exhaust(&mut self, amount: f32) {
        if amount > 0.0
            && let Some(p) = self.game.player.as_mut()
        {
            p.cause_food_exhaustion(amount);
        }
    }
}
impl Game<'_> {
    pub(super) fn veinminer_break_block(&mut self, pos: Position) -> Result<()> {
        with_state(self, |game, state| {
            if state.config.is_none() {
                state.config = Some(load_config().unwrap_or_else(|error| {
                    eprintln!("Veinminer disabled: {error}");
                    let mut config = Config::default();
                    config.settings.enabled = false;
                    config
                }));
            }
            let State { runtime, config, pending } = state;
            runtime
                .mine(&mut Native { game, pending: Some(pending) }, config.as_ref().unwrap(), pos)
                .map(|_| ())
        })
    }
    pub(super) fn veinminer_speed_multiplier(&mut self, pos: Position, speed: f32) -> f32 {
        with_state(self, |game, state| {
            if state.config.is_none() {
                state.config = Some(load_config().unwrap_or_else(|error| {
                    eprintln!("Veinminer disabled: {error}");
                    let mut config = Config::default();
                    config.settings.enabled = false;
                    config
                }));
            }
            let config = state.config.as_ref().unwrap();
            let mut host = Native { game, pending: None };
            let tool = host.tool();
            let Some(block) = host.block(pos) else { return speed };
            let Some((settings, targets)) = config.action(
                &block.name,
                tool.as_ref().map(|t| t.item.as_str()),
            ) else {
                return speed;
            };
            if !settings.enabled {
                return speed;
            }
            let vein_size = engine::preview(&host, pos, &targets, &settings).len();
            let time_factor =
                1.0 + settings.mining_speed_modifier * vein_size.saturating_sub(1) as f32;
            speed * settings.mining_speed_multiplier * (1.0 / time_factor).max(0.01)
        })
    }

    pub(super) fn veinminer_redirect_drop(&mut self, stack: &horizon_client::items::ItemStack, position: [f64; 3], velocity: [f64; 3], pickup_delay: u32) -> core::ops::ControlFlow<()> {
        let target = DROP_TARGET.with(|slot| *slot.borrow());
        let Some(target) = target else { return core::ops::ControlFlow::Continue(()); };
        let Some(_guard) = RedirectGuard::enter() else { return core::ops::ControlFlow::Continue(()); };
        self.spawn_item(stack.clone(), target, velocity, pickup_delay);
        let _ = position;
        core::ops::ControlFlow::Break(())
    }
    pub(super) fn veinminer_tick(&mut self) -> Result<()> {
        with_state(self, |game, state| {
            if state.config.is_none() {
                state.config = Some(load_config().unwrap_or_else(|error| {
                    eprintln!("Veinminer disabled: {error}");
                    let mut config = Config::default();
                    config.settings.enabled = false;
                    config
                }));
            }
            let actor = game.acting_player_uuid();
            let now = game.world.game_time;
            let mut due = Vec::new();
            let mut waiting = VecDeque::new();
            while let Some(task) = state.pending.pop_front() {
                if task.due_tick <= now && task.actor == actor { due.push(task); }
                else { waiting.push_back(task); }
            }
            state.pending = waiting;
            let config = state.config.as_ref().unwrap().clone();
            for task in due {
                let mut host = Native { game, pending: None };
                let player = host.player();
                if !player.survival || !state.runtime.enabled(task.actor) { continue; }
                let Some(block) = host.block(task.position) else { continue; };
                if block != task.expected { continue; }
                let tool = host.tool();
                if task.tool_item.as_ref().is_some_and(|id| tool.as_ref().is_none_or(|tool| &tool.item != id)) { continue; }
                let Some((settings, _)) = config.action(&block.name, tool.as_ref().map(|tool| tool.item.as_str())) else { continue; };
                if !settings.enabled
                    || settings.must_sneak && !player.sneaking
                    || settings.permission_restricted && !config.players.contains(&player.name)
                    || settings.require_enchantment && host.enchantment_level("veinminer_enchantment:veinminer") < settings.min_enchant_level
                    || settings.need_correct_tool && !host.can_harvest(&block)
                    || settings.decrease_durability && tool.as_ref().and_then(|tool| tool.remaining).is_some_and(|n| n <= 1)
                    || settings.hunger_per_block > 0.0 && !host.has_food()
                { continue; }
                if host.mine_with_drop_target(task.position, task.preserve_durability, task.drop_target)? {
                    host.exhaust(task.exhaustion);
                }
            }
            Ok(())
        })
    }
    pub(super) fn veinminer_command(&mut self, command: &str) -> Option<bool> {
        with_state(self, |game, state| {
            let mut words = command.split_whitespace();
            if words.next()? != "veinminer" {
                return None;
            }
            let args: Vec<_> = words.collect();
            let actor = game.acting_player_uuid();
            let message = match args.as_slice() {
                [] | ["status"] => format!(
                    "Veinminer: {}",
                    if state.runtime.enabled(actor) {
                        "on"
                    } else {
                        "off"
                    }
                ),
                ["toggle"] | ["on"] | ["off"] => {
                    let on = match args[0] {
                        "on" => true,
                        "off" => false,
                        _ => !state.runtime.enabled(actor),
                    };
                    state.runtime.set_enabled(actor, on);
                    format!("Veinminer: {}", if on { "on" } else { "off" })
                }
                ["reload"] => {
                    if game.ui.registries.permission < 2 {
                        game.command_failure(horizon_client::text::Text::literal(
                            "Veinminer reload requires permission level 2",
                        ));
                        return Some(false);
                    }
                    match load_config() {
                        Ok(config) => {
                            state.config = Some(config);
                            "Veinminer configuration reloaded".into()
                        }
                        Err(error) => {
                            game.command_failure(horizon_client::text::Text::literal(error));
                            return Some(false);
                        }
                    }
                }
                _ => {
                    game.command_failure(horizon_client::text::Text::literal(
                        "Usage: /veinminer [status|toggle|on|off|reload]",
                    ));
                    return Some(false);
                }
            };
            game.system_message(horizon_client::text::Text::literal(message));
            Some(true)
        })
    }
}

#[cfg(test)]
mod engine_tests {
    use super::engine as horizon_veinminer;
    include!("veinminer_tests.rs");
}
