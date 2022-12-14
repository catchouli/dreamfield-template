use bevy_ecs::component::Component;
use bevy_ecs::system::{Res, Query};
use cgmath::{Matrix3, Rad};
use dreamfield_system::resources::SimTime;
use dreamfield_system::components::Transform;

/// The fire orb component
#[derive(Component)]
pub struct FireOrb {
}

impl Default for FireOrb {
    fn default() -> Self {
        Self {}
    }
}

/// The fire orb movement system
pub fn fire_orb_movement(sim_time: Res<SimTime>, mut query: Query<(&mut FireOrb, &mut Transform)>)
{
    for (_, mut transform) in query.iter_mut() {
        let ball_height = sim_time.sim_time.sin() as f32 + 2.0;
        transform.pos.y = ball_height;
        transform.rot = Matrix3::from_angle_y(Rad(sim_time.sim_time as f32));
    }
}
