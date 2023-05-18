#![allow(clippy::type_complexity)]

use bevy_time::{prelude::*, TimePlugin};
use valence::network::ConnectionMode;
use valence::prelude::*;

mod attack;
mod chat;

fn main() {
    println!("Spleef startup");
    let mut net = NetworkSettings::default();
    net.connection_mode = ConnectionMode::Offline;
    App::new()
        .insert_resource(net)
        .add_plugins(DefaultPlugins)
        .add_plugin(TimePlugin)
        .add_plugin(chat::ChatPlugin)
        .add_plugin(attack::AttackPlugin)
        .add_startup_system(setup)
        .add_system(init_clients)
        .add_system(despawn_disconnected_clients)
        .add_system(dig)
        .insert_resource(RoundTimer(Timer::from_seconds(30.0, TimerMode::Repeating)))
        .add_system(round_timer)
        .run();
}

fn reset_map(instance: &mut Instance) {
    for y in 0..10 {
        for z in -25..25 {
            for x in -25..25 {
                instance.set_block([x, y * 10, z], BlockState::SNOW_BLOCK);
            }
        }
    }
}

fn setup(
    mut commands: Commands,
    server: Res<Server>,
    dimensions: Query<&DimensionType>,
    biomes: Query<&Biome>,
) {
    let mut instance = Instance::new(ident!("overworld"), &dimensions, &biomes, &server);
    for z in -5..5 {
        for x in -5..5 {
            instance.insert_chunk([x, z], Chunk::default());
        }
    }
    reset_map(&mut instance);
    commands.spawn(instance);
}

fn init_clients(
    mut clients: Query<(&mut Client, &mut Location, &mut Position, &mut GameMode), Added<Client>>,
    instances: Query<Entity, With<Instance>>,
) {
    for (mut client, mut loc, mut pos, mut game_mode) in &mut clients {
        *game_mode = GameMode::Survival;
        loc.0 = instances.single();
        pos.set([0.0, 128.0, 0.0]);

        client.send_message("SPLEEF SPLEEF SPLEEF".bold());
    }
}

fn dig(mut instances: Query<&mut Instance>, mut events: EventReader<Digging>) {
    let mut instance = instances.single_mut();

    for event in events.iter() {
        if event.state == DiggingState::Start {
            instance.set_block(event.position, BlockState::AIR);
        }
    }
}

#[derive(Resource)]
struct RoundTimer(Timer);

fn round_timer(
    time: Res<Time>,
    mut timer: ResMut<RoundTimer>,
    mut instances: Query<&mut Instance>,
    mut clients: Query<(&mut Client, &mut Position)>,
) {
    let mut instance = instances.single_mut();
    if timer.0.tick(time.delta()).just_finished() {
        reset_map(&mut instance);
        for (mut client, mut pos) in &mut clients {
            client.send_message("Resetting map.");
            pos.set([0.0, 256.0, 0.0]);
        }
    }
}
