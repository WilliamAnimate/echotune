// this crappy TUI engine is very high overhead; the code, in debug, has a catastrophically high,
// 0.0% CPU usage. This was done on an AMD A4-6210 with AMD Radeon R3 Graphics (4) @ 1.80 GHz.

#![allow(unused_must_use)]

use std::io::{stdout, StdoutLock, BufWriter, Write};
use std::sync::atomic::Ordering::Relaxed;
use crate::SONG_INDEX;
use echotune::RenderMode;

macro_rules! not_enough_space {
    ($tooey:expr) => {{
        $tooey.render_set_mode(RenderMode::NoSpace);
        // forgive me for this unfortunate error message.
        return Err(std::io::Error::new(std::io::ErrorKind::Unsupported, "s-stop!!~ there's not enough room... mmmfph"));
    }}
}

pub struct Tui<'a> {
    handle: BufWriter<StdoutLock<'a>>,
    rendering_mode: RenderMode,

    width: u16,
    height: u16,
    scrolling_offset: usize,
    pub cursor_index_queue: usize,
}

impl Tui<'_> {
    /// creates and primes the Tooey type, which... does the tui stuff
    ///
    /// it is recommended to create this on another thread.
    pub fn init() -> Tui<'static> {
        // lock stdout for perf; no other component should write directly there.
        // panic! writes to stderr, and can be captured via redirection (2>)
        let stdout = stdout().lock();
        // to avoid excessive syscalls (which yields the current thread and requires a context
        // switch, so increases overhead on the system itself), we buffer the stdout.
        let handle = BufWriter::new(stdout);

        Tui {
            handle,
            rendering_mode: RenderMode::Uninitialized,
            width: 0,
            height: 0,
            scrolling_offset: 0,
            cursor_index_queue: 0,
        }
    }

    fn determine_terminal_size(&mut self) {
        use terminal_size::{Width, Height, terminal_size};

        let (Width(width), Height(height)) = terminal_size().unwrap();
        self.width = width;
        self.height = height;
    }

    pub fn render_set_mode(&mut self, mode: RenderMode) {
        self.rendering_mode = mode;
    }

    pub fn tick(&mut self) {
        let time = std::time::Instant::now();
        self.rerender_display();
        writeln!(self.handle, "time taken to draw last frame: {:?}", time.elapsed());
        self.handle.flush();
    }

    pub fn rerender_display(&mut self) {
        self.__pre_rerender_display();
        if let Err(err) = self.__rerender_display() {
            if self.rendering_mode == RenderMode::NoSpace {
                self.rerender_display(); // rerender the nospace view right now, instead of waiting 1s
            } else {
                eprintln!("Unrecoginized error: {err}");
            }
        }
    }

    /// tasks that should be run before redrawing.
    /// this is important to make sure everything will draw correctly, however, the values it
    /// checks usually won't change unless the user is doing something. eg. resize terminal
    fn __pre_rerender_display(&mut self) {
        self.determine_terminal_size();
        self.__calculate_offset(); // N.B. this must be ran after determine_terminal_size();
                                   // otherwise, you risk a panic.
        self.cursor_index_queue = SONG_INDEX.load(Relaxed);
    }

    fn __rerender_display(&mut self) -> Result<(), std::io::Error> {
        match self.rendering_mode {
            RenderMode::Full => {
                self.__draw_full()?;
            },
            RenderMode::Safe => {
                self.__draw_safe()?;
            },
            RenderMode::NoSpace => {
                self.__draw_not_enough_space()?;
            }
            RenderMode::Uninitialized => panic!("Invalid state: rendering_mode is Uninitialized. Did you forget to call .render_set_mode?"),
        }

        // self.handle.flush()?;

        Ok(())
    }

    fn __calculate_offset(&mut self) {
        if self.cursor_index_queue >= self.height as usize - 12 + self.scrolling_offset {
            self.scrolling_offset += 1;
        }
        else if self.cursor_index_queue <= self.scrolling_offset {
            self.scrolling_offset = self.cursor_index_queue;
        }
    }

    fn __draw_full(&mut self) -> Result<(), std::io::Error> {
        let songs = &crate::PLAYLIST.read();

        if self.cursor_index_queue >= songs.len() {
            // wrap back to the size of songs; the user is trying to access songs.len() + 1
            // will panic otherwise, but callers dont need to care
            self.cursor_index_queue = songs.len() - 1;
            SONG_INDEX.store(self.cursor_index_queue, Relaxed);
        } else if self.scrolling_offset >= songs.len() {
            self.scrolling_offset = songs.len() - 1;
        }
        // TODO: put this in __pre_rerender_display
        self.__blankout_terminal();
        writeln!(self.handle, "current song index: {}, SONG_INDEX: {}, len: {}", self.cursor_index_queue, SONG_INDEX.load(Relaxed), songs.len());
        self.handle.flush();
        // TODO: make this only calculate once in determine_terminal_size, when size changes?
        let opening_box = self.draw_box::<true>("queue", self.width);
        let closing_box = self.draw_box::<false>("", self.width);
        let opening_box1 = self.draw_box::<true>("", self.width);
        let closing_box2 = self.draw_box::<false>("asdadsad", self.width);

        writeln!(self.handle, "{opening_box}");

        // HACK: for some reason, this code thinks cursor_index_queue^self.scrolling_offset is the
        // currently selected song. subtract it now.
        // i will give you a hug if you find out why that is, and a workaround that isn't this ugly.
        self.cursor_index_queue -= self.scrolling_offset;
        for i in 0..(self.height as usize - 12) + self.scrolling_offset {
            if i > songs.len() {
                break;
            }
            if i < self.scrolling_offset {
                continue;
            }
            if i >= songs.len() {
                // TODO: fill in the rest of the spaces with nothing? this should be an impossible
                // case unless i plan on adding `z` from vim
                break; // we've drawn all playlist entries. will panic otherwise (and UB in C)
            }

            let line = songs[i + self.scrolling_offset].split("/").last().unwrap_or("");
            #[allow(unused_assignments)] let mut entry: String = String::with_capacity(self.width.into());
            if i == self.cursor_index_queue {
                entry = self.draw_highlighted_entry(line)?;
            } else {
                entry = self.draw_entry(line)?;
            }
            write!(self.handle, "{entry}");
        }
        write!(self.handle, "{closing_box}");

        let line = songs[self.cursor_index_queue + self.scrolling_offset].split("/").last().unwrap_or("");
        let line = self.draw_entry_centered(line)?;
        // playback bar
        write!(self.handle, "{opening_box1}");
        write!(self.handle, "{line}");
        write!(self.handle, "{closing_box2}");
        writeln!(self.handle, "{}, {}", self.scrolling_offset, self.cursor_index_queue);

        Ok(())
    }

    fn __blankout_terminal(&mut self) {
        write!(self.handle, "\x1b[2J\x1b[H"); // top left corner; clear screen
    }

    fn __draw_safe(&mut self) -> Result<(), std::io::Error> {
        let songs = &crate::PLAYLIST.read();
        if self.cursor_index_queue >= songs.len() {
            self.cursor_index_queue = songs.len() - 1;
            SONG_INDEX.store(self.cursor_index_queue, Relaxed);
        }
        let song = songs[self.cursor_index_queue].split("/").last().unwrap_or("");

        self.__blankout_terminal();
        writeln!(self.handle, "{song}");
        let current_len = format_time(crate::SONG_CURRENT_LEN.load(Relaxed));
        let total_len = format_time(crate::SONG_TOTAL_LEN.load(Relaxed));
        let vol = f32_to_percent(crate::VOLUME_LEVEL.load(Relaxed));
        writeln!(self.handle, "{current_len} / {total_len}");
        writeln!(self.handle, "󰕾 {vol}%");
        self.handle.flush();

        Ok(())
    }

    fn __draw_not_enough_space(&mut self) -> Result<(), std::io::Error> {
        self.__blankout_terminal();
        writeln!(self.handle, "Echotune Error\n")?;
        writeln!(self.handle, "Not enough space for the terminal!")?;
        writeln!(self.handle, "Resize your terminal in order to see the queue. Keyboard input is still functional.")?;
        writeln!(self.handle, "To suppress this message, enter rm -rf /* in another shell session running under UID0 (root).")?;
        self.render_set_mode(RenderMode::Full); // TODO: change this to know what was there
                                                // previously

        Ok(())
    }

    pub fn enter_alt_buffer(&mut self) -> Result<(), std::io::Error> {
        writeln!(self.handle, "\x1B[?1049h")?;
        Ok(())
    }

    pub fn leave_alt_buffer(&mut self) -> Result<(), std::io::Error> {
        writeln!(self.handle, "\x1B[?1049l")?;
        Ok(())
    }

    fn draw_entry_centered(&mut self, text: &str) -> Result<String, std::io::Error> {
        let padding = 0;

        let pad_len = match self.width.checked_sub(text.len().try_into().unwrap()) {
            Some(n) => {
                match n.checked_sub(2) {
                    Some(n) => (n / 2) as usize,
                    None => not_enough_space!(self),
                }
            },
            None => not_enough_space!(self),
        };
        let mut ntext = String::with_capacity((self.width - 2).into());

        // :(
        // to see why this is here, run this on a terminal whose width is 84 chars with the song
        // name:
        // /home/william/Desktop/echotune_audio/badapple.mp3
        // TODO: get rid of this somehow
        if text.len() % 2 == 0 && self.width % 2 == 0 {
            ntext.push_str(&" ".repeat(pad_len - 2));
        } else {
            ntext.push_str(&" ".repeat(pad_len));
        }
        // put this here to hopefully center the text if both self.width and text.len's remainders
        // after a division of 2 equal 0
        if text.len() % 2 == 0 {
            ntext.push(' ');
        }
        ntext.push_str(text);
        ntext.push_str(&" ".repeat(pad_len));
        if self.width % 2 == 0 {
            ntext.push(' ');
        }

        Ok(box_draw_entry(&ntext, padding))
    }

    fn draw_entry(&mut self, text: &str) -> Result<String, std::io::Error> {
        let width = self.width as usize;
        let padding = width.checked_sub(text.len() + 2);
        if padding.is_none() {
            not_enough_space!(self);
        }
        Ok(box_draw_entry(text, padding.unwrap()))
    }

    fn draw_highlighted_entry(&mut self, text: &str) -> Result<String, std::io::Error> {
        let width = self.width as usize;
        let padding = width.checked_sub(text.len() + 2);
        if padding.is_none() {
            not_enough_space!(self);
        }

        let out = format!("\x1B[48;2;245;194;231m\x1B[38;2;30;30;46m{text}");
        Ok(box_draw_entry(&out, padding.unwrap()))
    }

    /// false for opening, true for closing
    fn draw_box<const CLOSING: bool>(&mut self, text: &str, term_len: u16) -> String /* aw man. */ {
        // this code is a piece of shit
        // TODO: refactor this
        let first: &str;
        let adding: u16;
        let closing: &str;
        let output: String;
        let trailing: String;
        if CLOSING {
            first = "╭─";
            adding = 3;
            closing = "╮";
            trailing = "─".repeat(((term_len - adding) - text.len() as u16).into());
            output = first.to_owned() + text + &trailing + closing;
        } else {
            first = "╰";
            adding = 2;
            closing = "╯";
            trailing = "─".repeat((term_len - adding).into());
            output = first.to_owned() + &trailing + closing;
        }

        output
    }
}

impl Drop for Tui<'_> {
    fn drop(&mut self) {
        self.leave_alt_buffer().unwrap();
    }
}

fn format_time(t: u64) -> String {
    let hrs =   t / 3600;
    let mins = (t % 3600) / 60;
    let secs =  t % 60;

    if hrs == 0 {
        format!("{:02}:{:02}", mins, secs)
    } else {
        format!("{:02}:{:02}:{:02}", hrs, mins, secs)
    }
}

/// nah not really you need to append the % yourself
fn f32_to_percent(f: f32) -> f32 {
    (f * 100.0).trunc()
}

fn box_draw_entry(text: &str, padding: usize) -> String {
    format!("│{}{}{}", text, &" ".repeat(padding), "\x1B[0m│")
}

