#[derive(PartialEq, Eq, Debug)]
pub enum SongControl {
    VolumeUp,
    VolumeDown,
    SeekForward,
    SeekBackward,

    ToggleLoop,
    PrevSong,
    NextSong,
    TogglePause,

    No, // skull

    DestroyAndExit,

    Unset,
}

#[derive(PartialEq, Debug)]
pub enum RenderMode {
    Safe, // if term is too small, or if under resource constraints, or user specified, or
    Full, // the entire TUI
    Reading, // loading playlist
    NoSpace,
    Uninitialized
}

// pub fn read_playlist() -> Vec<String> {
// }

