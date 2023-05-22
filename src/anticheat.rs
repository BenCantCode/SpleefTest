use bevy_time::{prelude::*, Stopwatch};
use valence::client::event_loop::PacketEvent;
use valence::client::ClientPlugin;
use valence::entity::Velocity;
use valence::glam::{dvec3, ivec3, vec3};
use valence::packet::c2s::play::{Full, PlayerInputC2s, PositionAndOnGround};
use valence::prelude::*;
pub struct AnticheatPlugin;

impl Plugin for AnticheatPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(on_join);
        app.add_system(update_last_grounded);
        app.add_system(anticheat);
    }
}

#[derive(Component)]
struct LastGrounded(Stopwatch);

#[derive(Component)]
struct LastPos(DVec3);

fn on_join(mut commands: Commands, players: Query<(Entity, &Position), Added<Client>>) {
    for (player, pos) in players.iter() {
        commands
            .entity(player)
            .insert(LastGrounded(Stopwatch::new()))
            .insert(LastPos(pos.0.into()));
    }
}

fn update_last_grounded(time: Res<Time>, mut components: Query<&mut LastGrounded>) {
    for mut component in components.iter_mut() {
        component.0.tick(time.delta());
    }
}

fn anticheat(
    time: Res<Time>,
    instances: Query<&Instance>,
    mut packets: EventReader<PacketEvent>,
    mut clients: Query<(&mut Client, &mut LastPos, &mut LastGrounded, &Location)>,
) {
    for raw in packets.iter() {
        let (position, on_ground) = if let Some(pkt) = raw.decode::<PositionAndOnGround>() {
            (pkt.position, pkt.on_ground)
        } else if let Some(pkt) = raw.decode::<Full>() {
            (pkt.position, pkt.on_ground)
        } else {
            continue;
        };
        let Ok((mut client, mut last_pos_component, mut last_grounded, location)) = clients.get_mut(raw.client) else {
            continue;
        };
        let Ok(instance) = instances.get(location.0) else {
            continue;
        };
        let last_pos = last_pos_component.0;
        last_pos_component.0 = position;

        let plausibly_grounded = 'grounded_test: {
            for y in -1..=0 {
                for x in -1..=1 {
                    for z in -1..=1 {
                        let block =
                            instance.block((position.as_ivec3() + ivec3(x, y, z)).to_array());
                        if let Some(block) = block {
                            if !block.state().is_air() {
                                //TODO: Check x and y of collision boxes.
                                if block.state().collision_shapes().any(|shape| {
                                    let max_y =
                                        (position.y + (y as f64) + 0.01).floor() + shape.max_y;
                                    // println!("Comparing {} > {}", max_y, position.y - 0.33);
                                    max_y > position.y - 0.33
                                }) {
                                    // println!("Grounded!");
                                    break 'grounded_test true;
                                }
                            } else {
                                // println!("air");
                            }
                        } else {
                            // println!("no block");
                        }
                    }
                }
            }
            false
        };

        if plausibly_grounded {
            // println!("grounded!");
            last_grounded.0.reset();
            continue;
        } else {
            // println!("not grounded!");
        }

        if on_ground {
            // println!("on ground cap");
        }

        // println!("last grounded: {:?}", last_grounded.0.elapsed_secs());
        if !on_ground && (last_pos.y > position.y || last_grounded.0.elapsed_secs() < 0.5) {
            continue;
        }
        println!("hax");
        client.send_message("hax!!!");
    }
}
