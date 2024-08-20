use std::io::{self, Read};
use std::os::unix::io::AsRawFd;
use termios::*;

// #[allow(unused)]
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

        let mut ret: echotune::SongControl = Unset;
        let mut buffer = [0; 1];
        let b = self.handle.read(&mut buffer).unwrap();
        // dbg!(b);
        while b == 1 {
            // println!("?");
            let byte = buffer[0];

            // ctrl+c
            if byte == 3 {
                ret = DestroyAndExit;
                break;
            }

            // special keys
            // TODO: emums
            if byte == 27 {
                // escape sequence for arrow keys starts with 27 (<esc>)
                self.handle.read_exact(&mut buffer).unwrap();
                if buffer[0] == 91 {
                    self.handle.read_exact(&mut buffer).unwrap();
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
                ret = match byte {
                    114 => ToggleLoop,
                    72 => PrevSong,
                    76 => NextSong,
                    32 => TogglePause,
                    _ => No,
                }
            }

            if ret != Unset {
                break;
            }
        }

        // if ret == SongControl::Unset {
        //     println!("Note: SongControl is Unset!");
        //     ret = SongControl::No;
        // }

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

