use std::ops::Mul;

use valence::{
    entity::{EntityId, Velocity},
    glam::vec3,
    packet::{
        c2s::play::player_interact_entity::EntityInteraction,
        encode::WritePacket,
        s2c::play::{DamageTiltS2c, EntityAnimationS2c, EntityDamageS2c},
    },
    prelude::*,
};

pub struct AttackPlugin;

impl Plugin for AttackPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(punch);
    }
}

fn punch(
    mut clients: Query<(&mut Client, &Look, &Velocity, &EntityId, &Position)>,
    mut interact_entity: EventReader<InteractEntity>,
) {
    let mut packets: Vec<EntityDamageS2c> = vec![];
    for event in interact_entity.iter() {
        if event.interact != EntityInteraction::Attack {
            continue;
        }
        let attacker = clients.get(event.client).unwrap();
        let attacker_look = attacker.1.vec();
        let attacker_look_yaw = attacker.1.yaw;
        let attacker_id = attacker.3.get();
        let attacker_pos = attacker.4 .0;
        let mut victim = clients.get_mut(event.entity).unwrap();
        let victim_velocity: Vec3 = victim.2 .0;
        // Add velocity to victim
        victim
            .0
            .set_velocity(attacker_look.mul(20.0) + vec3(0.0, 8.0, 0.0) + victim_velocity);
        // Send victim a packet (with id 0).
        victim.0.write_packet(&EntityDamageS2c {
            entity_id: 0.into(),
            source_type_id: (1).into(), /* ENTITY_ATTACK */
            source_cause_id: (attacker_id + 1).into(),
            source_direct_id: (attacker_id + 1).into(),
            source_pos: Some(attacker_pos),
        });
        // Create damage packet
        packets.push(EntityDamageS2c {
            entity_id: victim.3.get().into(),
            source_type_id: (1).into(), /* ENTITY_ATTACK */
            source_cause_id: (attacker_id + 1).into(),
            source_direct_id: (attacker_id + 1).into(),
            source_pos: Some(attacker_pos),
        });
    }
    for mut client in clients.iter_mut() {
        for packet in &packets {
            // Victim has already received packet.
            if packet.entity_id.0 != client.3.get() {
                client.0.write_packet(packet);
            }
        }
    }
}
