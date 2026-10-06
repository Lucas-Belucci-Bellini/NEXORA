//! The player's own save section (PLAYER-53, PLAYER-54; ADR-0036).
//!
//! `PLAYER SYSTEM.md` asks for the player's *location* to be saved
//! (PLAYER-53), and for the save to be split by domain rather than kept as
//! one *"Player.json com tudo"* (PLAYER-54). So the player is its own section
//! of the save container, beside the world's three (`nexora_world::persist`),
//! and the world's sections neither know nor need it: a world saved without a
//! player still loads, and one loaded without reading this section is the
//! same world.
//!
//! What it holds is [`PlayerState`] — the feet, the velocity, whether the
//! body stood on something, and the facing — because that is everything
//! [`Player::tick`] reads: a player resumed from it ([`Player::resume`])
//! runs, tick for tick, the ticks the saved one would have run, to the bit.
//! Floats are stored as their bits; nothing is rounded on the way through.
//!
//! What it does not hold: an identity, an inventory, anything of PLAYER-54's
//! other domains. The player is not an entity yet (DEBT-0050), so there is
//! one, and no id to give it.
//!
//! # Format, version 1
//!
//! ```text
//! version     u16   1
//! feet        3 × f64 (x, y, z)
//! velocity    3 × f64
//! grounded    u8    0 or 1
//! yaw, pitch  2 × f64
//! ```
//!
//! Little-endian, through the persistence crate's bounds-checked codec. A
//! save is untrusted input (`NEXORA SECURITY THREAT MODEL.md`): a section
//! with a version this build does not know, a truncated or over-long body, a
//! flag that is not 0 or 1, or a value no player can have is refused by name
//! rather than turned into a player somewhere strange.

use nexora_foundation::error::{Domain, Error, Recovery, Result};
use nexora_foundation::ident::Identifier;
use nexora_foundation::spatial::WorldPosition;
use nexora_persistence::codec::{Reader, Writer};
use nexora_persistence::container::SaveContainer;

use crate::player::{Player, PlayerState};

/// The section the player is saved in.
pub const SECTION_PLAYER: &str = "nexora:save/player";

/// The version of the section this build writes, and the only one it reads.
pub const PLAYER_SECTION_VERSION: u16 = 1;

/// The section's length in bytes at version 1.
const SECTION_LEN: usize = 2 + 8 * 3 + 8 * 3 + 1 + 8 * 2;

/// The player's section, encoded.
#[must_use]
pub fn encode_player(state: &PlayerState) -> Vec<u8> {
    let mut writer = Writer::new();
    writer.u16(PLAYER_SECTION_VERSION);
    for value in [state.feet.x, state.feet.y, state.feet.z] {
        writer.f64(value);
    }
    for value in state.velocity {
        writer.f64(value);
    }
    writer.u8(u8::from(state.grounded));
    writer.f64(state.yaw);
    writer.f64(state.pitch);
    writer.finish()
}

/// Read the player's section.
///
/// Values are returned as stored; whether they are a state a player can be
/// in is [`Player::resume`]'s to judge, against the terrain.
///
/// # Errors
///
/// [`Recovery::Reject`] when the version is unknown, the section is not
/// exactly as long as its version says, or the ground flag is not 0 or 1.
pub fn decode_player(bytes: &[u8]) -> Result<PlayerState> {
    let mut reader = Reader::new(bytes);
    let version = reader.u16()?;
    if version != PLAYER_SECTION_VERSION {
        return Err(
            refused("the player section has a version this build does not read")
                .with_context("version", version.to_string())
                .with_context("supported", PLAYER_SECTION_VERSION.to_string()),
        );
    }
    if bytes.len() != SECTION_LEN {
        return Err(
            refused("the player section is not the length its version says")
                .with_context("length", bytes.len().to_string())
                .with_context("expected", SECTION_LEN.to_string()),
        );
    }
    let feet = WorldPosition::new(reader.f64()?, reader.f64()?, reader.f64()?);
    let velocity = [reader.f64()?, reader.f64()?, reader.f64()?];
    let grounded = match reader.u8()? {
        0 => false,
        1 => true,
        other => {
            return Err(refused("the player section's ground flag is not 0 or 1")
                .with_context("flag", other.to_string()))
        }
    };
    let (yaw, pitch) = (reader.f64()?, reader.f64()?);
    Ok(PlayerState {
        feet,
        velocity,
        grounded,
        yaw,
        pitch,
    })
}

/// Put `player` into `container`, replacing any player section it held.
///
/// # Errors
///
/// The section name is not a valid identifier (a defect, not a state).
pub fn save_player(container: &mut SaveContainer, player: &Player) -> Result<()> {
    container.put(
        Identifier::parse(SECTION_PLAYER)?,
        encode_player(&player.state()),
    );
    Ok(())
}

/// The player `container` holds, if it holds one.
///
/// # Errors
///
/// The section is present and refused by [`decode_player`].
pub fn load_player(container: &SaveContainer) -> Result<Option<PlayerState>> {
    container
        .get(&Identifier::parse(SECTION_PLAYER)?)
        .map(decode_player)
        .transpose()
}

fn refused(message: &'static str) -> Error {
    Error::new(Domain::Save, "player-persist", message).with_recovery(Recovery::Reject)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::player::{Walk, MAX_PITCH};
    use crate::spawn::{find_walkable_run, ColumnArea};
    use nexora_physics::voxel::FlatGround;

    const TPS: u32 = 20;

    fn state() -> PlayerState {
        PlayerState {
            feet: WorldPosition::new(-12.25, 64.0, 7.0 / 3.0),
            velocity: [0.1, -9.80665, -0.0],
            grounded: false,
            yaw: -2.5,
            pitch: -0.25,
        }
    }

    #[test]
    fn a_state_survives_its_section_to_the_bit() {
        let bytes = encode_player(&state());
        assert_eq!(bytes.len(), SECTION_LEN);
        let back = decode_player(&bytes).unwrap();
        assert!(back.is_bit_identical(&state()), "{back:?}");
        assert!(back.velocity[2].is_sign_negative(), "-0.0 kept its sign");
    }

    #[test]
    fn a_damaged_section_is_refused_by_name() {
        let good = encode_player(&state());
        let mut future = good.clone();
        future[0] = 2;
        assert!(decode_player(&future)
            .unwrap_err()
            .to_string()
            .contains("version"));
        assert!(decode_player(&good[..good.len() - 1]).is_err(), "truncated");
        let mut long = good.clone();
        long.push(0);
        assert!(decode_player(&long)
            .unwrap_err()
            .to_string()
            .contains("length"));
        let mut flag = good.clone();
        flag[2 + 48] = 7;
        assert!(decode_player(&flag)
            .unwrap_err()
            .to_string()
            .contains("ground flag"));
        assert!(decode_player(&[]).is_err());
    }

    #[test]
    fn a_container_without_a_player_has_none() {
        let mut container = SaveContainer::new();
        assert_eq!(load_player(&container).unwrap(), None);
        container.put(
            Identifier::parse(SECTION_PLAYER).unwrap(),
            encode_player(&state()),
        );
        let decoded = SaveContainer::decode(&container.encode()).unwrap();
        let back = load_player(&decoded).unwrap().unwrap();
        assert!(back.is_bit_identical(&state()));
    }

    /// The point of keeping everything a tick reads: a player resumed from
    /// its section mid-walk and mid-jump runs the same ticks as the one that
    /// was saved, to the bit.
    #[test]
    fn a_resumed_player_runs_the_ticks_the_saved_one_would_have() {
        let ground = FlatGround::at(0);
        let area = ColumnArea {
            min_x: -16,
            min_z: -16,
            max_x: 16,
            max_z: 16,
            floor_y: -8,
            ceiling_y: 8,
        };
        let run = find_walkable_run(&ground, &area, 4).unwrap();
        let mut original = Player::spawn(&ground, &run, 0.0, TPS).unwrap();
        let walk = Walk {
            forward: 1.0,
            right: 0.5,
            turn: 0.3,
            look: -0.2,
            jump: false,
        };
        for _ in 0..7 {
            original.tick(&ground, walk).unwrap();
        }
        // Mid-jump: off the ground and rising.
        original.tick(&ground, Walk { jump: true, ..walk }).unwrap();
        assert!(!original.state().grounded && original.state().velocity[1] > 0.0);

        let saved = decode_player(&encode_player(&original.state())).unwrap();
        let mut resumed = Player::resume(&ground, &saved, TPS).unwrap();
        assert!(resumed.state().is_bit_identical(&original.state()));
        for tick in 0..60 {
            let walk = Walk {
                jump: tick == 30,
                ..walk
            };
            original.tick(&ground, walk).unwrap();
            resumed.tick(&ground, walk).unwrap();
            assert!(
                resumed.state().is_bit_identical(&original.state()),
                "tick {tick}: {:?} against {:?}",
                resumed.state(),
                original.state()
            );
        }
    }

    /// Standing still and resumed, a jump on the very first tick is a jump:
    /// whether the body stood on something is part of what a tick reads.
    #[test]
    fn a_resumed_player_standing_can_jump_at_once() {
        let ground = FlatGround::at(0);
        let area = ColumnArea {
            min_x: -16,
            min_z: -16,
            max_x: 16,
            max_z: 16,
            floor_y: -8,
            ceiling_y: 8,
        };
        let run = find_walkable_run(&ground, &area, 4).unwrap();
        let mut original = Player::spawn(&ground, &run, 0.0, TPS).unwrap();
        assert!(original.state().grounded);
        let saved = decode_player(&encode_player(&original.state())).unwrap();
        let mut resumed = Player::resume(&ground, &saved, TPS).unwrap();
        let jump = Walk {
            jump: true,
            ..Walk::default()
        };
        original.tick(&ground, jump).unwrap();
        resumed.tick(&ground, jump).unwrap();
        assert!(original.state().velocity[1] > 0.0, "the original jumped");
        assert!(resumed.state().is_bit_identical(&original.state()));
    }

    #[test]
    fn a_player_cannot_be_resumed_into_terrain_or_out_of_range() {
        let ground = FlatGround::at(0);
        let standing = PlayerState {
            feet: WorldPosition::new(0.5, 0.0, 0.5),
            velocity: [0.0; 3],
            grounded: true,
            yaw: 0.0,
            pitch: 0.0,
        };
        assert!(Player::resume(&ground, &standing, TPS).is_ok());
        let buried = PlayerState {
            feet: WorldPosition::new(0.5, -0.5, 0.5),
            ..standing
        };
        assert!(Player::resume(&ground, &buried, TPS)
            .unwrap_err()
            .to_string()
            .contains("inside terrain"));
        for strange in [
            PlayerState {
                pitch: MAX_PITCH * 1.01,
                ..standing
            },
            PlayerState {
                yaw: -std::f64::consts::PI,
                ..standing
            },
            PlayerState {
                feet: WorldPosition::new(f64::NAN, 0.0, 0.5),
                ..standing
            },
            PlayerState {
                velocity: [0.0, f64::INFINITY, 0.0],
                ..standing
            },
        ] {
            assert!(
                Player::resume(&ground, &strange, TPS).is_err(),
                "{strange:?}"
            );
        }
    }
}
