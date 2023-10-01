// Keys to avoid in defaults:
//  - Ctrl - ctrl+W closes the browser tab
//  - Alt - shows/hides the firefox menu bar on linux
//  - Numpad - Some keyboards might not have it
//  - Keys that often depend on layout - https://github.com/not-fl3/macroquad/issues/260
// LATER Configurable input

use macroquad::prelude::*;

use crate::prelude::*;

/// The complete state of client-side input in one frame.
///
/// Not all of this is sent to the server because not all of it is relevant to the server.
#[derive(Clone, Copy, Default)]
pub struct ClientInput {
    pub left: bool,
    pub right: bool,
    pub up: bool,
    pub down: bool,
    pub turret_left: bool,
    pub turret_right: bool,
    pub prev_weapon: bool,
    pub next_weapon: bool,
    pub fire: bool,
    pub mine: bool,
    pub self_destruct: bool,
    pub horn: bool,
    pub chat: bool,
    pub pause: bool,
    // ^ when adding fields, also add them to Debug
}

/// Subset of ClientInput relevant to the server.
///
/// LATER Include stuff like timestamps.
/// LATER Maybe treat some keys presses as events?
#[derive(Clone, Copy, Default, Serialize, Deserialize)]
pub struct NetInput {
    pub left: bool,
    pub right: bool,
    pub up: bool,
    pub down: bool,
    pub turret_left: bool,
    pub turret_right: bool,
    pub prev_weapon: bool,
    pub next_weapon: bool,
    pub fire: bool,
    pub mine: bool,
    pub self_destruct: bool,
    pub horn: bool,
    // ^ when adding fields, also add them to Debug
}

// LATER Restore or remove State/Events. Probably use bitflags for events.

// /// Keys held this frame.
// #[derive(Clone, Copy, Default, Serialize, Deserialize)]
// pub struct InputState {
//     pub left: bool,
//     pub right: bool,
//     pub up: bool,
//     pub down: bool,
//     pub horn: bool,
//     // ^ when adding fields, also add them to Debug
// }

// /// Keys pressed this frame.
// #[derive(Debug, Clone, Copy, Serialize, Deserialize)]
// pub enum InputEvent {
//     TurretLeft,
//     TurretRight,
//     PrevWeapon,
//     NextWeapon,
//     Fire,
//     Mine,
//     SelfDestruct,
//     Pause,
// }

impl ClientInput {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn merged(&self, other: ClientInput) -> Self {
        Self {
            left: self.left | other.left,
            right: self.right | other.right,
            up: self.up | other.up,
            down: self.down | other.down,
            turret_left: self.turret_left | other.turret_left,
            turret_right: self.turret_right | other.turret_right,
            prev_weapon: self.prev_weapon | other.prev_weapon,
            next_weapon: self.next_weapon | other.next_weapon,
            fire: self.fire | other.fire,
            mine: self.mine | other.mine,
            self_destruct: self.self_destruct | other.self_destruct,
            horn: self.horn | other.horn,
            chat: self.chat | other.chat,
            pause: self.pause | other.pause,
        }
    }

    pub fn to_net_input(&self) -> NetInput {
        NetInput {
            left: self.left,
            right: self.right,
            up: self.up,
            down: self.down,
            turret_left: self.turret_left,
            turret_right: self.turret_right,
            prev_weapon: self.prev_weapon,
            next_weapon: self.next_weapon,
            fire: self.fire,
            mine: self.mine,
            self_destruct: self.self_destruct,
            horn: self.horn,
        }
    }
}

impl NetInput {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn new_up() -> Self {
        Self {
            up: true,
            ..Self::default()
        }
    }

    pub fn right_left(&self) -> f64 {
        self.right as i32 as f64 - self.left as i32 as f64
    }

    pub fn up(&self) -> f64 {
        self.up as i32 as f64
    }

    pub fn down(&self) -> f64 {
        self.down as i32 as f64
    }

    /// Subset of inputs to control the missile
    pub fn missile_while_guiding(&self) -> Self {
        Self {
            up: true,
            down: false,
            ..*self
        }
    }

    /// Subset of inputs to control the vehicle while guiding a missile
    pub fn vehicle_while_guiding(&self) -> Self {
        // Original RW allowed everything except movement.
        Self {
            left: false,
            right: false,
            up: false,
            down: false,
            ..*self
        }
    }
}

impl Debug for ClientInput {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        static_assert!(
            std::mem::size_of::<ClientInput>() == 14,
            "number of fields changed without changing Debug impl"
        );

        write!(f, "Input {{ ")?;
        if self.left {
            write!(f, "left ")?;
        }
        if self.right {
            write!(f, "right ")?;
        }
        if self.up {
            write!(f, "up ")?;
        }
        if self.down {
            write!(f, "down ")?;
        }
        if self.turret_left {
            write!(f, "turret_left ")?;
        }
        if self.turret_right {
            write!(f, "turret_right ")?;
        }
        if self.prev_weapon {
            write!(f, "prev_weapon ")?;
        }
        if self.next_weapon {
            write!(f, "next_weapon ")?;
        }
        if self.fire {
            write!(f, "fire ")?;
        }
        if self.mine {
            write!(f, "mine ")?;
        }
        if self.self_destruct {
            write!(f, "self_destruct ")?;
        }
        if self.horn {
            write!(f, "horn ")?;
        }
        if self.chat {
            write!(f, "chat ")?;
        }
        if self.pause {
            write!(f, "pause ")?;
        }
        write!(f, "}}")?;
        Ok(())
    }
}

impl Debug for NetInput {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        static_assert!(
            std::mem::size_of::<NetInput>() == 12,
            "number of fields changed without changing Debug impl"
        );

        write!(f, "NetInput {{ ")?;
        if self.left {
            write!(f, "left ")?;
        }
        if self.right {
            write!(f, "right ")?;
        }
        if self.up {
            write!(f, "up ")?;
        }
        if self.down {
            write!(f, "down ")?;
        }
        if self.turret_left {
            write!(f, "turret_left ")?;
        }
        if self.turret_right {
            write!(f, "turret_right ")?;
        }
        if self.prev_weapon {
            write!(f, "prev_weapon ")?;
        }
        if self.next_weapon {
            write!(f, "next_weapon ")?;
        }
        if self.fire {
            write!(f, "fire ")?;
        }
        if self.mine {
            write!(f, "mine ")?;
        }
        if self.self_destruct {
            write!(f, "self_destruct ")?;
        }
        if self.horn {
            write!(f, "horn ")?;
        }
        write!(f, "}}")?;
        Ok(())
    }
}

// impl Debug for InputState {
//     fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
//         static assert
//
//         write!(f, "InputState {{ ")?;
//         if self.left {
//             write!(f, "left ")?;
//         }
//         if self.right {
//             write!(f, "right ")?;
//         }
//         if self.up {
//             write!(f, "up ")?;
//         }
//         if self.down {
//             write!(f, "down ")?;
//         }
//         if self.horn {
//             write!(f, "horn ")?;
//         }
//         write!(f, "}}")?;
//         Ok(())
//     }
// }

pub fn get_input1() -> ClientInput {
    let mut input = ClientInput::empty();
    if was_input_pressed(&[KeyCode::A]) {
        input.left = true;
    }
    if was_input_pressed(&[KeyCode::D]) {
        input.right = true;
    }
    if was_input_pressed(&[KeyCode::W]) {
        input.up = true;
    }
    if was_input_pressed(&[KeyCode::S]) {
        input.down = true;
    }
    if was_input_pressed(&[KeyCode::Q]) {
        input.turret_left = true;
    }
    if was_input_pressed(&[KeyCode::E]) {
        input.turret_right = true;
    }
    if was_input_pressed(&[KeyCode::V]) {
        input.prev_weapon = true;
    }
    if was_input_pressed(&[KeyCode::LeftShift, KeyCode::C]) {
        input.next_weapon = true;
    }
    if was_input_pressed(&[KeyCode::Space]) {
        input.fire = true;
    }
    if was_input_pressed(&[KeyCode::X]) {
        input.mine = true;
    }
    if was_input_pressed(&[KeyCode::G]) {
        input.self_destruct = true;
    }
    if was_input_pressed(&[KeyCode::R]) {
        input.horn = true;
    }

    // The rest are shared actions defined on is player 1 only

    if was_input_pressed(&[KeyCode::Enter, KeyCode::T]) {
        input.chat = true;
    }
    if was_input_pressed(&[KeyCode::Pause, KeyCode::P]) {
        input.pause = true;
    }

    input
}

pub fn get_input2() -> ClientInput {
    let mut input = ClientInput::empty();
    if was_input_pressed(&[KeyCode::Left]) {
        input.left = true;
    }
    if was_input_pressed(&[KeyCode::Right]) {
        input.right = true;
    }
    if was_input_pressed(&[KeyCode::Up]) {
        input.up = true;
    }
    if was_input_pressed(&[KeyCode::Down]) {
        input.down = true;
    }
    if was_input_pressed(&[KeyCode::Comma]) {
        input.turret_left = true;
    }
    if was_input_pressed(&[KeyCode::Period]) {
        input.turret_right = true;
    }
    if was_input_pressed(&[KeyCode::L]) {
        input.prev_weapon = true;
    }
    if was_input_pressed(&[
        KeyCode::Slash, // US layout
        KeyCode::Minus, // Same key, CZ layout
        KeyCode::Kp0,
    ]) {
        input.next_weapon = true;
    }
    if was_input_pressed(&[KeyCode::RightShift]) {
        input.fire = true;
    }
    if was_input_pressed(&[KeyCode::M]) {
        input.mine = true;
    }
    if was_input_pressed(&[KeyCode::J]) {
        input.self_destruct = true;
    }
    if was_input_pressed(&[KeyCode::K]) {
        input.horn = true;
    }

    // No binds for shared actions like chat, pause, console and esc.
    // They're defined on player 1.

    input
}

fn was_input_pressed(key_codes: &[KeyCode]) -> bool {
    for &key_code in key_codes {
        // Check both to avoid skipping input if it's pressed and released within one frame.
        if is_key_pressed(key_code) || is_key_down(key_code) {
            return true;
        }
    }
    false
}
