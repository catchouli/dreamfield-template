mod player_movement;
mod fire_orb;
mod entity_spawner;
mod minecart;

pub use player_movement::*;
pub use fire_orb::*;

use bevy_ecs::schedule::SystemSet;

// Sim systems
pub fn systems() -> SystemSet {
    SystemSet::new()
        .label("sim")
        .with_system(entity_spawner::entity_spawner)
        .with_system(player_movement::player_update)
        .with_system(fire_orb::fire_orb_movement)
        .with_system(minecart::update_minecart)
}
