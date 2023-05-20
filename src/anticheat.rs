use bevy_time::{prelude::*, Stopwatch};
use valence::client::event_loop::PacketEvent;
use valence::client::ClientPlugin;
use valence::entity::Velocity;
use valence::glam::{dvec3, ivec3, vec3};
use valence::packet::c2s::play::{PlayerInputC2s, PositionAndOnGround};
use valence::prelude::*;
pub struct AnticheatPlugin;

impl Plugin for AnticheatPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(on_join);
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

fn anticheat(
    time: Res<Time>,
    mut packets: EventReader<PacketEvent>,
    mut clients: Query<(&mut Client, &mut LastPos, &mut LastGrounded)>,
    instances: Query<&Instance>,
) {
    let instance = instances.single();
    for raw in packets.iter() {
        if let Some(pkt) = raw.decode::<PositionAndOnGround>() {
            let (mut client, mut last_pos_component, mut last_grounded) =
                clients.get_mut(raw.client).unwrap();
            let last_pos = last_pos_component.0;
            last_pos_component.0 = pkt.position;

            let plausibly_grounded = 'grounded_test: {
                for y in -2..=-1 {
                    for x in -1..=1 {
                        for z in -1..=1 {
                            let block = instance
                                .block((pkt.position.as_ivec3() + ivec3(x, y, z)).to_array());
                            if let Some(block) = block {
                                if !block.state().is_air() {
                                    break 'grounded_test true;
                                }
                            }
                        }
                    }
                }
                false
            };

            if plausibly_grounded {
                last_grounded.0.reset();
            }

            if pkt.on_ground {
                if plausibly_grounded {
                    return;
                }
            } else {
                if last_pos.y > pkt.position.y
                    || last_grounded.0.tick(time.delta()).elapsed_secs() < 0.5
                {
                    return;
                }
            }
            println!("hax");
            client.send_message("hax!!!");
        }
    }
}
