/// Structure keeping track of button pushes
/// 1 -> left button, 2 -> right button
pub struct ButtonsState {
    pub button_mask: u8,
    pub cmd_buffer: [u8; 4],
}

impl Default for ButtonsState {
    fn default() -> Self {
        ButtonsState {
            button_mask: 0,
            cmd_buffer: [0u8; 4],
        }
    }
}

impl ButtonsState {
    pub fn new() -> ButtonsState {
        ButtonsState::default()
    }
}

/// Event types needed by
/// an application
pub enum ButtonEvent {
    LeftButtonPress,
    RightButtonPress,
    BothButtonsPress,
    LeftButtonRelease,
    RightButtonRelease,
    BothButtonsRelease,
}

/// Distinguish between button press and button release
pub fn get_button_event(buttons: &mut ButtonsState, new: u8) -> Option<ButtonEvent> {
    let old = buttons.button_mask;
    buttons.button_mask |= new;
    match (old, new) {
        (0, 1) => Some(ButtonEvent::LeftButtonPress),
        (0, 2) => Some(ButtonEvent::RightButtonPress),
        (_, 3) => Some(ButtonEvent::BothButtonsPress),
        (b, 0) => {
            buttons.button_mask = 0; // reset state on release
            match b {
                1 => Some(ButtonEvent::LeftButtonRelease),
                2 => Some(ButtonEvent::RightButtonRelease),
                3 => Some(ButtonEvent::BothButtonsRelease),
                _ => None,
            }
        }
        _ => None,
    }
}
