/// Structure keeping track of button pushes
/// 1 -> left button, 2 -> right button
#[derive(Default)]
pub struct ButtonsState {
    pub button_mask: u8,
    pub cmd_buffer: [u8; 4],
}

impl ButtonsState {
    pub const fn new() -> ButtonsState {
        ButtonsState {
            button_mask: 0,
            cmd_buffer: [0; 4],
        }
    }
}

/// Event types needed by
/// an application
#[derive(Eq, PartialEq)]
#[repr(u8)]
pub enum ButtonEvent {
    LeftButtonPressed = BUTTON_LEFT_PRESSED as u8, 
    RightButtonPressed = BUTTON_RIGHT_PRESSED as u8,            
    LeftButtonConitnuousPressed = BUTTON_LEFT_CONTINUOUS_PRESSED as u8,   
    RightButtonConitnuousPressed = BUTTON_RIGHT_CONTINUOUS_PRESSED as u8,
    BothButtonsPressed = BUTTON_BOTH_PRESSED as u8,
    BothButtonTouched = BUTTON_BOTH_TOUCHED as u8, 
    InvalidButtonEvent = INVALID_BUTTON_EVENT as u8
}

impl from<u8> for ButtonEvent {
    fn from(v: u8) -> ButtonEvent {
        match v {
            BUTTON_LEFT_PRESSED => ButtonEvent::LeftButtonPressed,
            BUTTON_RIGHT_PRESSED => ButtonEvent::RightButtonPressed,
            BUTTON_LEFT_CONTINUOUS_PRESSED => ButtonEvent::LeftButtonConitnuousPressed,
            BUTTON_RIGHT_CONTINUOUS_PRESSED => ButtonEvent::RightButtonConitnuousPressed,
            BUTTON_BOTH_PRESSED => ButtonEvent::BothButtonsPressed,
            BUTTON_BOTH_TOUCHED => ButtonEvent::BothButtonTouched,
            _ => ButtonEvent::InvalidButtonEvent,
        }
    }
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
