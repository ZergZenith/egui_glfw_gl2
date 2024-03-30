use egui::{CursorIcon, Key, Modifiers};
use glfw::Modifiers as Mod;
use winapi::ctypes::wchar_t;
use winapi::um::winuser;

pub fn translate_modifiers(keymod: Mod) -> Modifiers {
    Modifiers {
        alt: keymod & Mod::Alt == Mod::Alt,
        ctrl: keymod & Mod::Control == Mod::Control,
        shift: keymod & Mod::Shift == Mod::Shift,
        command: keymod & Mod::Control == Mod::Control,
        // TODO: GLFW doesn't seem to support the mac command key
        //       mac_cmd: keymod & Mod::LGUIMOD == Mod::LGUIMOD,
        mac_cmd: false,
    }
}

pub fn is_cut_command(modifiers: Modifiers, keycode: glfw::Key) -> bool {
    (modifiers.command && keycode == glfw::Key::X)
        || (cfg!(target_os = "windows")
        && modifiers.shift
        && keycode == glfw::Key::Delete)
}

pub fn is_copy_command(modifiers: Modifiers, keycode: glfw::Key) -> bool {
    (modifiers.command && keycode == glfw::Key::C)
        || (cfg!(target_os = "windows")
        && modifiers.ctrl
        && keycode == glfw::Key::Insert)
}

pub fn is_paste_command(modifiers: Modifiers, keycode: glfw::Key) -> bool {
    (modifiers.command && keycode == glfw::Key::V)
        || (cfg!(target_os = "windows")
        && modifiers.shift
        && keycode == glfw::Key::Insert)
}

pub fn translate_virtual_key_code(key: glfw::Key) -> Option<Key> {
    use glfw::Key::*;

    Some(match key {
        Left => Key::ArrowLeft,
        Up => Key::ArrowUp,
        Right => Key::ArrowRight,
        Down => Key::ArrowDown,

        Escape => Key::Escape,
        Tab => Key::Tab,
        Backspace => Key::Backspace,
        Space => Key::Space,

        Enter => Key::Enter,

        Insert => Key::Insert,
        Home => Key::Home,
        Delete => Key::Delete,
        End => Key::End,
        PageDown => Key::PageDown,
        PageUp => Key::PageUp,

        A => Key::A,
        B => Key::B,
        C => Key::C,
        D => Key::D,
        E => Key::E,
        F => Key::F,
        G => Key::G,
        H => Key::H,
        I => Key::I,
        J => Key::J,
        K => Key::K,
        L => Key::L,
        M => Key::M,
        N => Key::N,
        O => Key::O,
        P => Key::P,
        Q => Key::Q,
        R => Key::R,
        S => Key::S,
        T => Key::T,
        U => Key::U,
        V => Key::V,
        W => Key::W,
        X => Key::X,
        Y => Key::Y,
        Z => Key::Z,

        _ => {
            return None;
        }
    })
}


pub fn translate_cursor(cursor_icon: CursorIcon) -> Option<WinCursorIcon> {
    match cursor_icon {
        CursorIcon::None => None,

        CursorIcon::Alias => Some(WinCursorIcon::Alias),
        CursorIcon::AllScroll => Some(WinCursorIcon::AllScroll),
        CursorIcon::Cell => Some(WinCursorIcon::Cell),
        CursorIcon::ContextMenu => Some(WinCursorIcon::ContextMenu),
        CursorIcon::Copy => Some(WinCursorIcon::Copy),
        CursorIcon::Crosshair => Some(WinCursorIcon::Crosshair),
        CursorIcon::Default => Some(WinCursorIcon::Default),
        CursorIcon::Grab => Some(WinCursorIcon::Grab),
        CursorIcon::Grabbing => Some(WinCursorIcon::Grabbing),
        CursorIcon::Help => Some(WinCursorIcon::Help),
        CursorIcon::Move => Some(WinCursorIcon::Move),
        CursorIcon::NoDrop => Some(WinCursorIcon::NoDrop),
        CursorIcon::NotAllowed => Some(WinCursorIcon::NotAllowed),
        CursorIcon::PointingHand => Some(WinCursorIcon::Hand),
        CursorIcon::Progress => Some(WinCursorIcon::Progress),

        CursorIcon::ResizeHorizontal => Some(WinCursorIcon::EwResize),
        CursorIcon::ResizeNeSw => Some(WinCursorIcon::NeswResize),
        CursorIcon::ResizeNwSe => Some(WinCursorIcon::NwseResize),
        CursorIcon::ResizeVertical => Some(WinCursorIcon::NsResize),

        CursorIcon::ResizeEast => Some(WinCursorIcon::EResize),
        CursorIcon::ResizeSouthEast => Some(WinCursorIcon::SeResize),
        CursorIcon::ResizeSouth => Some(WinCursorIcon::SResize),
        CursorIcon::ResizeSouthWest => Some(WinCursorIcon::SwResize),
        CursorIcon::ResizeWest => Some(WinCursorIcon::WResize),
        CursorIcon::ResizeNorthWest => Some(WinCursorIcon::NwResize),
        CursorIcon::ResizeNorth => Some(WinCursorIcon::NResize),
        CursorIcon::ResizeNorthEast => Some(WinCursorIcon::NeResize),
        CursorIcon::ResizeColumn => Some(WinCursorIcon::ColResize),
        CursorIcon::ResizeRow => Some(WinCursorIcon::RowResize),

        CursorIcon::Text => Some(WinCursorIcon::Text),
        CursorIcon::VerticalText => Some(WinCursorIcon::VerticalText),
        CursorIcon::Wait => Some(WinCursorIcon::Wait),
        CursorIcon::ZoomIn => Some(WinCursorIcon::ZoomIn),
        CursorIcon::ZoomOut => Some(WinCursorIcon::ZoomOut),
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum WinCursorIcon {
    /// The platform-dependent default cursor.
    Default,
    /// A simple crosshair.
    Crosshair,
    /// A hand (often used to indicate links in web browsers).
    Hand,
    #[allow(dead_code)]
    Arrow,
    /// Indicates something is to be moved.
    Move,
    /// Indicates text that may be selected or edited.
    Text,
    /// Program busy indicator.
    Wait,
    /// Help indicator (often rendered as a "?")
    Help,
    /// Progress indicator. Shows that processing is being done. But in contrast
    /// with "Wait" the user may still interact with the program. Often rendered
    /// as a spinning beach ball, or an arrow with a watch or hourglass.
    Progress,

    /// Cursor showing that something cannot be done.
    NotAllowed,
    ContextMenu,
    Cell,
    VerticalText,
    Alias,
    Copy,
    NoDrop,
    /// Indicates something can be grabbed.
    Grab,
    /// Indicates something is grabbed.
    Grabbing,
    AllScroll,
    ZoomIn,
    ZoomOut,

    /// Indicate that some edge is to be moved. For example, the 'SeResize' cursor
    /// is used when the movement starts from the south-east corner of the box.
    EResize,
    NResize,
    NeResize,
    NwResize,
    SResize,
    SeResize,
    SwResize,
    WResize,
    EwResize,
    NsResize,
    NeswResize,
    NwseResize,
    ColResize,
    RowResize,
}

impl Default for WinCursorIcon {
    fn default() -> Self {
        WinCursorIcon::Default
    }
}

impl WinCursorIcon {
    pub(crate) fn to_windows_cursor(self) -> *const wchar_t {
        match self {
            WinCursorIcon::Arrow | WinCursorIcon::Default => winuser::IDC_ARROW,
            WinCursorIcon::Hand => winuser::IDC_HAND,
            WinCursorIcon::Crosshair => winuser::IDC_CROSS,
            WinCursorIcon::Text | WinCursorIcon::VerticalText => winuser::IDC_IBEAM,
            WinCursorIcon::NotAllowed | WinCursorIcon::NoDrop => winuser::IDC_NO,
            WinCursorIcon::Grab | WinCursorIcon::Grabbing | WinCursorIcon::Move | WinCursorIcon::AllScroll => {
                winuser::IDC_SIZEALL
            }
            WinCursorIcon::EResize
            | WinCursorIcon::WResize
            | WinCursorIcon::EwResize
            | WinCursorIcon::ColResize => winuser::IDC_SIZEWE,
            WinCursorIcon::NResize
            | WinCursorIcon::SResize
            | WinCursorIcon::NsResize
            | WinCursorIcon::RowResize => winuser::IDC_SIZENS,
            WinCursorIcon::NeResize | WinCursorIcon::SwResize | WinCursorIcon::NeswResize => {
                winuser::IDC_SIZENESW
            }
            WinCursorIcon::NwResize | WinCursorIcon::SeResize | WinCursorIcon::NwseResize => {
                winuser::IDC_SIZENWSE
            }
            WinCursorIcon::Wait => winuser::IDC_WAIT,
            WinCursorIcon::Progress => winuser::IDC_APPSTARTING,
            WinCursorIcon::Help => winuser::IDC_HELP,
            _ => winuser::IDC_ARROW, // use arrow for the missing cases.
        }
    }
}
