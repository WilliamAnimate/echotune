use std::io::{self, Read};
// i hate windows with a passion
use std::os::unix::io::AsRawFd;
use termios::*;

pub struct Input<'a> {
    handle: io::StdinLock<'a>,
    fd: i32,
    original_terminal_config: Termios,
}

impl<'a> Input<'_> {
    /// this must be mutable!
    pub fn from_nothing_and_apply() -> Input<'a> {
        let stdin = io::stdin();
        let handle = stdin.lock();
        let fd = handle.as_raw_fd();

        // current terminal. save it to restore the terminal upon exiting
        let original_termios = Termios::from_fd(fd).unwrap();

        // we need a mutable copy of the terminal settings, to mess around with
        let mut raw = original_termios;

        // disable echoing of user input
        raw.c_lflag &= !(ECHO | ICANON | IEXTEN | ISIG);

        // apply
        tcsetattr(fd, TCSANOW, &raw).unwrap();

        Input {
            handle,
            fd,
            original_terminal_config: original_termios,
        }
    }

    pub fn blocking_wait_for_input(&mut self) -> echotune::SongControl {
        use echotune::SongControl::*;

        let mut ret: echotune::SongControl;
        let mut buffer = [0; 1];
        let b = self.handle.read(&mut buffer).unwrap();
        debug_assert!(b == 1, "1 byte not read");
        loop {
            let byte = buffer[0];

            // ctrl+c (byte == 3 is end of text)
            if byte == 3 {
                ret = DestroyAndExit;
                break;
            }

            // byte 27 is esc (\x1B)
            if byte == 27 {
                self.handle.read_exact(&mut buffer).unwrap();
                // 91 is [
                if buffer[0] == 91 {
                    /*
                     * if we get to this point, it means we got `\x1B[`
                     * how? im glad you didn't ask:
                     * byte == 27: `\x1B` (aka <esc>)
                     * buffer[0] == 91: `[`
                     * that ultimately gives us `\x1B[`
                     * by that point, if you want to do up arrow, you would get:
                     * `\x1B[A` (where A is keycode 65... more on that below...)
                     */
                    self.handle.read_exact(&mut buffer).unwrap();
                    /*
                     * 65 - up arrow
                     * 66 - down arrow
                     * 67 - right arrow
                     * 68 - left arrow
                     * these can be represented as a char, however, it might cause more confusion
                     * than good (because `A => VolumeUp` implies pressing A increases the volume
                     * at first glance)
                     */
                    ret = match buffer[0] {
                        65 => VolumeUp,
                        66 => VolumeDown,
                        67 => SeekForward,
                        68 => SeekBackward,
                        _ => No,
                    };
                } else {
                    ret = No; // no paths here.
                }
            } else {
                let char = dtoc(byte);
                ret = match char {
                    'r' => ToggleLoop,
                    'k' => PrevSong,
                    'j' => NextSong,
                    ' ' => TogglePause,
                    _ => No,
                }
            }

            if ret != Unset {
                break;
            }
        }

        ret
    }

    pub fn restore_terminal(&self) -> Result<(), std::io::Error> {
        tcsetattr(self.fd, TCSANOW, &self.original_terminal_config)
    }
}

impl Drop for Input<'_> {
    fn drop(&mut self) {
        if let Err(err) = self.restore_terminal() {
            eprintln!("can't restore the terminal to its original state. THIS IS A BUG!\n{err}");
        }
    }
}

/// stands for **d**ec **to** **c**har
fn dtoc(i: u8) -> char {
    if i > 127 {
        // TODO: handle this in the impossible case it does occur.
        panic!("i ({i}) > 127!");
    }
    char::from_u32(i as u32).unwrap()
}

