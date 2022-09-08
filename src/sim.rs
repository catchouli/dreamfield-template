mod player_movement;
mod fire_orb;

pub use player_movement::*;
pub use fire_orb::*;

use bevy_ecs::schedule::SystemSet;

// Sim systems
pub fn systems() -> SystemSet {
    SystemSet::new()
        .label("sim")
        .with_system(player_movement::player_update)
        .with_system(fire_orb_movement)
}
