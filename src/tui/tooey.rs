// this crappy TUI engine is very "high" overhead; the code, in debug, has a catastrophically high,
// 0.0% CPU usage. This was done on an AMD A4-6210 with AMD Radeon R3 Graphics (4) @ 1.80 GHz.
// any performance improvements should be considered.

// N.B. Performance improvements come from reducing allocations. Do not premature optimize.
use std::io::{stdout, StdoutLock, BufWriter, Write};
use echotune::RenderMode;

macro_rules! not_enough_space {
    ($tooey:expr) => {{
        $tooey.render_set_mode(RenderMode::NoSpace);
        // forgive me for this unfortunate error message.
        return Err(std::io::Error::new(std::io::ErrorKind::Unsupported, "s-stop!!~ there's not enough room... mmmfph"));
    }}
}

#[derive(Debug)]
#[allow(unused)] // shut the fuck up
enum CursorLocation {
    Queue,
    Nya,
    NowPlaying,
    None
}

// #[derive(Debug)]
#[allow(unused)]
pub struct Tooey<'a> {
    handle: BufWriter<StdoutLock<'a>>,
    rendering_mode: RenderMode,

    width: u16,
    height: u16,
    playlist_len: u16, // don't anticipate playlist changing every second
    scrolling_offset: usize,
    cursor: CursorLocation,
    pub cursor_index_queue: u16,
}

#[allow(unused)] // shut the fuck up
impl Tooey<'_> {
    /// creates and primes the Tooey type, which... does the tui stuff
    ///
    /// it is recommended to create this on another thread.
    pub fn init() -> Tooey<'static> {
        // lock stdout for perf; no other component should write directly there.
        // panic! writes to stderr
        let stdout = stdout().lock();
        // to avoid excessive syscalls (which yields the current thread and requires a context
        // switch, so increases overhead on the system itself), we buffer the stdout.
        let handle = BufWriter::new(stdout);

        Tooey {
            handle,
            rendering_mode: RenderMode::Uninitialized,
            width: 0,
            height: 0,
            playlist_len: 0,
            scrolling_offset: 0,
            cursor: CursorLocation::None,
            cursor_index_queue: 31,
        }
    }

    pub fn set_playlist_len(&mut self, len: u16) {
        self.playlist_len = len;
    }

    // pub fn next_entry(&mut self) {
    //     self.adjust_cursor_queue(self.cursor_index_queue + 1);
    // }

    /// increment with tooey.cursor_index_queue + 1; decrement with tooey.cursor_index_queue - 1;
    pub fn adjust_cursor_queue(&mut self, n: u16) {
        // if self.cursor_index_queue >= self.playlist_len {
        //     self.cursor_index_queue = self.playlist_len as u16 - 1;
        //     return;
        // }
        eprintln!("{} {}", self.cursor_index_queue, self.playlist_len);
        self.cursor_index_queue = n;
    }

    fn determine_terminal_size(&mut self) -> Result<(), std::io::Error> {
        use terminal_size::{Width, Height, terminal_size};

        let (Width(width), Height(height)) = terminal_size().unwrap();
        self.width = width;
        self.height = height;

        Ok(())
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
            #[allow(unreachable_code)] _ => unreachable!("how'd we get here? bad RenderMode value."),
        }

        Ok(())
    }

    fn __draw_full(&mut self) -> Result<(), std::io::Error> {
        let songs = crate::PLAYLIST.clone();
        let songs = songs.read().unwrap(); // shadowing go brr; fuck lifetimes

        if self.cursor_index_queue as usize >= songs.len() {
            // wrap back to the size of songs; the user is trying to access songs.len() + 1
            // will panic otherwise, but callers dont need to care
            self.cursor_index_queue = songs.len() as u16 - 1;
        }
        self.__blankout_terminal();
        let opening_box = self.draw_box::<true>("queue", self.width);
        let closing_box = self.draw_box::<false>("", self.width);
        let opening_box1 = self.draw_box::<true>("", self.width);
        let closing_box2 = self.draw_box::<false>("asdadsad", self.width);
        writeln!(self.handle, "timings: {:?}", std::time::Instant::now())?;
        write!(self.handle, "{opening_box}");

        let mut c1 = false;
        let mut c2 = false;

        let mut index = 0;
        let mut starting_index = 0;
        // let mut offset = self.scrolling_offset;
        for _ in index..songs.len() {
            if index >= (self.height - 10).into() {
                // self.scrolling_offset -= 1;
                break;
            }
            if index == 0 {
                if self.cursor_index_queue >= self.height - 10 {
                    c1 = true;
                    if self.cursor_index_queue as usize != index {
                        c2 = true;
                        starting_index += 1;
                    }
                }
            }
            // else if self.cursor_index_queue.saturating_sub(self.height) == 0 {
            //     self.scrolling_offset -= 1;
            // }

            // let line = songs[index + self.scrolling_offset].split("/").last().unwrap_or("");
            let mut entry: String = Default::default();
            if starting_index == self.cursor_index_queue.into() {
                entry = self.draw_highlighted_entry(&format!("{}+{}={}; c1: {}, c2: {}", starting_index, self.scrolling_offset, index + self.scrolling_offset, c1, c2))?
                // entry = self.draw_highlighted_entry(line)?
            } else {
                entry = self.draw_entry(&format!("{}+{}={}; c1: {}, c2: {}", starting_index, self.scrolling_offset, index + self.scrolling_offset, c1, c2))?
                // entry = self.draw_entry(line)?
            };
            write!(self.handle, "{entry}");
            index += 1;
            c1 = false;
            c2 = false;
        }

        // for (mut index, song) in (*songs).iter().enumerate() {
        //     if index >= (self.height - 10).into() {
        //         // writeln!(self.handle, "{index}, {}", songs.len());
        //         // std::thread::sleep(std::time::Duration::from_secs(1));
        //         break;
        //     }
        //     if !first {
        //         index = 999;
        //         first = true;
        //     }
        //
        //     let line = song.split('/').last().unwrap_or("");
        //     let entry: String = if index == self.cursor_index_queue.into() {
        //         self.draw_highlighted_entry(line)?
        //     } else {
        //         self.draw_entry(line)?
        //     };
        //     write!(self.handle, "{entry}");
        //     index += 1;
        // }
        write!(self.handle, "{closing_box}");

        // playback bar
        write!(self.handle, "{opening_box1}");
        let currently_playing_song_name = &songs[self.cursor_index_queue as usize];
        let currently_playing_song_name = currently_playing_song_name.split('/').last().unwrap_or("");
        let now_playing = self.draw_entry_centered(&format!("now playing: {currently_playing_song_name}"))?;
        write!(self.handle, "{now_playing}");
        write!(self.handle, "{closing_box2}");

        self.handle.flush();

        Ok(())
    }

    fn __blankout_terminal(&mut self) {
        write!(self.handle, "\x1b[2J\x1b[H"); // top left corner; clear screen
    }

    fn __draw_safe(&mut self) -> Result<(), std::io::Error> {
        todo!("safe mode not implemented");
    }

    fn __draw_not_enough_space(&mut self) -> Result<(), std::io::Error> {
        self.__blankout_terminal();
        writeln!(self.handle, "Echotune Error\n")?;
        writeln!(self.handle, "Not enough space for the terminal!")?;
        writeln!(self.handle, "Resize your terminal in order to see the queue. Keyboard input is still functional.")?;
        writeln!(self.handle, "To suppress this message, enter rm -rf /* in another shell session running under UID0 (root).")?;
        self.handle.flush();
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

    fn __draw_entry(&mut self, text: &str, term_len: u16, padding: usize) -> String {
        format!("│{}{}{}", text, &" ".repeat(padding), "\x1B[0m│")
    }

    fn draw_entry_centered(&mut self, text: &str) -> Result<String, std::io::Error> {
        let width = self.width as usize;
        let padding = 0;

        let pad_len = match (self.width.checked_sub((text.len()).try_into().unwrap())) {
            Some(n) => {
                match n.checked_sub(2) {
                    Some(n) => (n / 2) as usize,
                    None => not_enough_space!(self),
                }
            },
            None => not_enough_space!(self),
        };
        // let dbg = self.draw_entry(&format!("post subtract: {pad_len} term len: {}, alloced {}", self.width, self.width - 2))?;
        // let dbg2 = self.draw_entry(&format!("text len: {}, text len %2: {}, term width %2: {}", text.len(), text.len() % 2, self.width % 2))?;
        // writeln!(self.handle, "{}\n{dbg2}", dbg);
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

        Ok(self.__draw_entry(&ntext, self.width, padding))
    }

    fn draw_entry(&mut self, text: &str) -> Result<String, std::io::Error> {
        let width = self.width as usize;
        let padding = width.checked_sub(text.len() + 2);
        if padding.is_none() {
            not_enough_space!(self);
        }
        Ok(self.__draw_entry(text, self.width, padding.unwrap()))
    }

    fn draw_highlighted_entry(&mut self, text: &str) -> Result<String, std::io::Error> {
        // \e[1;33;4;44m
        let width = self.width as usize;
        let padding = match width.checked_sub(text.len() + 2) {
            Some(padding) => padding,
            None => {
                not_enough_space!(self);
            }
        };

        let out = format!("\x1B[48;2;245;194;231m\x1B[38;2;30;30;46m{text}");
        Ok(self.__draw_entry(&out, self.width, padding))
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

    pub fn echotune_render_playlist(&mut self) {

    }
}

impl Drop for Tooey<'_> {
    fn drop(&mut self) {
        self.leave_alt_buffer().unwrap();
    }
}

