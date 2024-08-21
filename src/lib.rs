#[derive(PartialEq, Eq, Debug, Copy)]
/// don't Box<SongControl> this value, or you're going to have a very hard time with .clone()
/// because it will panic.
/// :troll:
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

impl Clone for SongControl {
    fn clone(&self) -> Self {
        panic!("why are we on the heap???");
    }
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

