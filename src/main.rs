mod song;
mod input;
mod tui;

use std::sync::{atomic::{AtomicBool, AtomicU16, Ordering::Relaxed}, mpsc::channel};
use parking_lot::RwLock;

macro_rules! send_control {
    ($signal:expr, $($tx:expr),*) => {
        $({
            $tx.send($signal)?
        })*
    }
}

macro_rules! __exit_await_thread {
    ($($thread:expr),*) => {
        $(
            $thread.join().unwrap();
        )*
    }
}

lazy_static::lazy_static!{
    static ref PLAYLIST: RwLock<Vec<String>> = Default::default();
    static ref CFG_IS_LOOPED: AtomicBool = AtomicBool::new(false);
    static ref SONG_INDEX: AtomicU16 = AtomicU16::new(0);
}

fn parse_playlist(file: &str) -> Result<(), Box<dyn std::error::Error>> {
    use std::{io::{BufReader, BufRead}, fs::File};

    let reader = BufReader::new(File::open(file)?);

    let mut lines = PLAYLIST.write();
    #[allow(deprecated)]
    let home = std::env::home_dir().unwrap().to_str().unwrap().to_string(); // its fine; we never running on NT
    for line in reader.lines() {
        let mut line = line.unwrap(); // tf
        // PERF: don't replace nothing and allocate a new String, unless we actually do start with ~
        // maybe. idfk. this only runs once as part of initialization.
        if line.starts_with('~') {
            line = line.replacen('~', &home, 1);
        } else if line.starts_with("//") {
            continue; // its a comment; skip
        }
        lines.push(line);
    }
    let _ = lines.pop(); // the last element is nothing, for some reason. get rid of it now.

    Ok(())
}

fn quit_with(e: &str, s: &str) -> Result<std::convert::Infallible, Box<dyn std::error::Error>> {
    eprintln!("{e}");
    return Err(s.into());
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::thread::spawn;
    use echotune::SongControl::*;
    use file_format::Kind;

    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        quit_with("argv[1] should be a media file or echotune-compatable playlist.", "argv[1] not supplied")?;
    }

    let file = &args[1];

    let _ = match file_format::FileFormat::from_file(file)?.kind() {
        Kind::Audio => {
            let mut lines = PLAYLIST.write();
            lines.push(file.to_string());
        },
        Kind::Other => parse_playlist(&file)?,
        filekind => {
            let _ = quit_with(&format!("argv[1] should be a media file or echotune-compatable playlist. media type of {filekind:?} is not supported."), "argv[1] unsupported")?;
        },
    };

    let (rtx, rrx) = channel();
    let render = spawn(move || {
        let mut tooey = tui::tooey::Tooey::init();
        tooey.render_set_mode(echotune::RenderMode::Full);
        tooey.enter_alt_buffer().unwrap();
        loop {
            tooey.tick();
//             debug_assert!(tooey.cursor_index_queue == SONG_INDEX.load(Relaxed),
//             "Inconsistent state: cursor_index_queue != SONG_INDEX! IOW, you've reached a serious synchronisation problem that could affect release mode.\n\
// good luck. you've messed up big time. {} != {}", tooey.cursor_index_queue, SONG_INDEX.load(Relaxed));
            let receive = rrx.try_recv();
            if let Ok(k) = receive {
                match k {
                    DestroyAndExit => break, // the destructor will exit the alt buffer
                    ToggleLoop => CFG_IS_LOOPED.store(!CFG_IS_LOOPED.load(Relaxed), Relaxed),
                    _na => {
                        #[cfg(debug_assertions)]
                        eprintln!("the operation {_na:?} is not applicable for rendering");
                    }
                };
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
    });

    let (atx, arx) = channel();
    let audio = spawn(move || {
        let mut audio = song::Song::new();
        audio.play();
        loop {
            let receive = arx.try_recv();
            if let Ok(k) = receive {
                match k {
                    DestroyAndExit => break,
                    PrevSong | NextSong => audio.rejitter_song(),
                    TogglePause => if audio.sink.is_paused() {audio.play()} else {audio.pause()} // why no ternary operator in rust
                    VolumeUp => {
                        let prev_vol = audio.sink.volume();
                        audio.sink.set_volume(prev_vol + 0.1);
                    },
                    VolumeDown => {
                        let prev_vol = audio.sink.volume();
                        let request_vol = prev_vol - 0.1;
                        // no .saturating_sub for f32 cause primitive type, so we do this:
                        let normalized_vol = if request_vol < 0.0 { 0.0 } else { request_vol };
                        audio.sink.set_volume(normalized_vol);
                    },
                    _na => {
                        #[cfg(debug_assertions)]
                        eprintln!("the operation {_na:?} is not applicable for audio");
                    }
                }
            }

            if audio.sink.empty() {
                if CFG_IS_LOOPED.load(Relaxed) {
                    // TODO: test this
                    audio.rejitter_song();
                } else {
                    SONG_INDEX.store(SONG_INDEX.load(Relaxed) + 1, Relaxed);
                    audio.rejitter_song();
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
    });

    let mut input = input::Input::from_nothing_and_apply();
    loop {
        let i = input.blocking_wait_for_input();
        match i {
            DestroyAndExit => {
                // send them sigterms'
                send_control!(DestroyAndExit, rtx, atx);

                // wait for the threads to finish
                __exit_await_thread!(render, audio);

                // restore terminal setttings (because i don't trust the destructor)
                input.restore_terminal()?;
                break;
            },
            NextSong => {
                SONG_INDEX.store(SONG_INDEX.load(Relaxed) + 1, Relaxed);
                send_control!(NextSong, rtx, atx);
            }
            PrevSong => {
                let sub = match SONG_INDEX.load(Relaxed).checked_sub(1) {
                    Some(n) => n,
                    None => continue,
                };
                SONG_INDEX.store(sub, Relaxed);
                send_control!(PrevSong, rtx, atx);
            }
            No => (), // there is nothing
            signal => {
                send_control!(signal, rtx, atx);
            }
        }
    }

    Ok(())
}

