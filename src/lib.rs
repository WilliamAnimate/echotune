#![forbid(unsafe_code)]

use std::sync::atomic::{AtomicUsize, Ordering};

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
    NoSpace,
    Uninitialized
}

#[derive(PartialEq, Debug)]
pub enum FileFormat {
    Audio,

    // and if no match
    Other
}

pub struct AtomicF32(AtomicUsize);

/// no hardware support bruh
impl AtomicF32 {
    #[inline] pub fn new(v: f32) -> Self {
        AtomicF32(AtomicUsize::new(v.to_bits().try_into().unwrap()))
    }

    #[inline] pub fn load(&self, order: Ordering) -> f32 {
        f32::from_bits(self.0.load(order).try_into().unwrap())
    }

    #[inline] pub fn store(&self, val: f32, order: Ordering) {
        self.0.store(val.to_bits().try_into().unwrap(), order);
    }
}

