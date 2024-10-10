mod song;
mod input;
mod tui;

use std::sync::{atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering::Relaxed}, mpsc::channel, Arc};
use parking_lot::RwLock;

macro_rules! send_control_errorless {
    ($signal:expr, $($tx:expr),*) => {
        $({
            let _ = $tx.send($signal);
        })*
    }
}

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
    static ref SONG_INDEX: AtomicUsize = AtomicUsize::new(0);
    static ref SONG_TOTAL_LEN: AtomicU64 = AtomicU64::new(0);
    static ref SONG_CURRENT_LEN: AtomicU64 = AtomicU64::new(0);
    static ref VOLUME_LEVEL: echotune::AtomicF32 = echotune::AtomicF32::new(0.0);
}

fn parse_playlist(file: &str) -> Result<(), Box<dyn std::error::Error>> {
    use std::{io::{BufReader, BufRead}, fs::File};

    let reader = BufReader::new(File::open(file)?);

    let mut lines = PLAYLIST.write();
    let home = std::env::var("HOME").unwrap_or_else(|_| String::new());
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
    Err(s.into())
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
    let mut render_requested_mode = echotune::RenderMode::Full;

    match file_format::FileFormat::from_file(file)?.kind() {
        Kind::Audio => {
            let mut lines = PLAYLIST.write();
            render_requested_mode = echotune::RenderMode::Safe; // only one song, so do minimal
            lines.push(file.to_string());
        },
        Kind::Other => parse_playlist(file)?,
        filekind => {
            let _ = quit_with(&format!("argv[1] should be a media file or echotune-compatable playlist. media type of {filekind:?} is not supported."), "argv[1] unsupported")?;
        },
    };

    let (mtx, mrx) = channel();
    let mtx = Arc::new(mtx);
    let audio_over_mtx = mtx.clone();
    let ctrlc_mtx = mtx.clone();

    let (rtx, rrx) = channel();
    let rtx = Arc::new(rtx);
    let main_rtx = rtx.clone();
    let render = spawn(move || {
        let mut tui = tui::Tui::init();
        tui.render_set_mode(render_requested_mode);
        tui.enter_alt_buffer().unwrap();
        loop {
            tui.tick();
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
    let atx = Arc::new(atx);
    let main_atx = atx.clone();
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
                    // seeking may fail. if so, then silently fail, because who cares??
                    SeekForward => {
                        let _ = audio.sink.try_seek(audio.sink.get_pos() + std::time::Duration::from_secs(5));
                    }
                    SeekBackward => {
                        let _ = audio.sink.try_seek(audio.sink.get_pos().saturating_sub(std::time::Duration::from_secs(5)));
                    }
                    _na => {
                        #[cfg(debug_assertions)]
                        eprintln!("the operation {_na:?} is not applicable for audio");
                    }
                }
            }

            if audio.sink.empty() {
                let song_index = SONG_INDEX.load(Relaxed);
                if CFG_IS_LOOPED.load(Relaxed) {
                    audio.rejitter_song();
                } else if (song_index as usize) > PLAYLIST.read().len() {
                    SONG_INDEX.store(SONG_INDEX.load(Relaxed) + 1, Relaxed);
                    audio.rejitter_song();
                } else {
                    send_control_errorless!(DestroyAndExit, audio_over_mtx);
                    break;
                }
            } else {
                // task: synchronise global variables based on what we have.

                // there is a bug here: sometimes, this returns None.
                // some mp3s work, but others don't. i dont know why precisely.
                let total_dur = match audio.total_duration {
                    Some(n) => n.as_secs(),
                    None => 0,
                };
                SONG_CURRENT_LEN.store(audio.sink.get_pos().as_secs(), Relaxed);
                SONG_TOTAL_LEN.store(total_dur, Relaxed);

                VOLUME_LEVEL.store(audio.sink.volume(), Relaxed);
            }

            std::thread::sleep(std::time::Duration::from_millis(50));
        }
    });

    let _input = spawn(move || {
        let mut input = input::Input::from_nothing_and_apply();
        loop {
            let i = input.blocking_wait_for_input();
            match i {
                DestroyAndExit => {
                    let _ = input.restore_terminal();
                    send_control_errorless!(DestroyAndExit, ctrlc_mtx);
                    break;
                },
                NextSong => {
                    if PLAYLIST.read().len() != 1 {
                        let i = SONG_INDEX.load(Relaxed);
                        SONG_INDEX.store(i + 1, Relaxed);
                        send_control_errorless!(NextSong, rtx, atx);
                    }
                }
                PrevSong => {
                    let sub = match SONG_INDEX.load(Relaxed).checked_sub(1) {
                        Some(n) => n,
                        None => continue,
                    };
                    SONG_INDEX.store(sub, Relaxed);
                    send_control_errorless!(PrevSong, rtx, atx);
                }
                No => (), // there is nothing
                signal => {
                    send_control_errorless!(signal, rtx, atx);
                }
            }
        }
    });

    loop {
        let recv = mrx.recv().unwrap();
        match recv {
            DestroyAndExit => {
                send_control!(DestroyAndExit, main_rtx, main_atx);

                // wait for the threads to finish
                // FIXME: input doesnt seem to work. it hangs.
                __exit_await_thread!(render, audio);

                break;
            }
            _ => (), // dont care
        };
    }

    Ok(())
}

