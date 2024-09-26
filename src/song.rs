use rodio::{OutputStreamHandle, OutputStream, Decoder, Sink};

#[allow(unused)]
pub struct Song {
    pub current_song_index: u16, // keep track of which songs are currently playing, for backtracking.
    // N.B. KEEP STREAM HANDLE HERE TO NOT DROP IT!
    // this is important for playing audio.
    _stream_handle: OutputStreamHandle,
    _stream: OutputStream,
    pub sink: Sink,
    pub current_source: Option<Decoder<std::io::BufReader<std::fs::File>>>,

    pub current_duration: Option<std::time::Duration>,
}

impl Song {
    pub fn new() -> Song {
        let (_stream, _stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&_stream_handle).unwrap();
        let mut s = Song {
            current_song_index: 0,
            _stream_handle,
            _stream,
            sink,
            current_source: None,
            current_duration: None,
        };

        s.append_song(0);
        s
    }

    fn reset(&mut self) {
        self.sink.stop();
    }

    /// this can silently fail if this is the first song.
    pub fn prev_song(&mut self) {
        self.reset();
        // dbg!(self.current_song_index);
        let checked = match self.current_song_index.checked_sub(1) {
            Some(k) => k,
            None => return,
        };
        self.current_song_index -= 1;
        self.append_song(checked);
        self.play();
    }

    /// readd current song
    pub fn current_song(&mut self) {
        self.reset();
        self.append_song(self.current_song_index);
        self.play();
    }

    pub fn next_song(&mut self) {
        self.reset();
        // dbg!(self.current_song_index);
        self.current_song_index += 1;
        self.append_song(self.current_song_index);
        self.play()
    }

    fn append_song(&mut self, index: u16) {
        use std::{fs::File, io::BufReader};

        let to_open = &crate::PLAYLIST.read().unwrap().to_vec();
        if index as usize >= to_open.len() {
            // wrap back to the size of the playlist; the user is trying to access playlist.len() + 1
            // will panic otherwise, but callers dont need to care.
            self.current_song_index = to_open.len() as u16 - 1;
            return;
        }
        let file = BufReader::new(File::open(&to_open[index as usize]).unwrap());
        let source = Decoder::new(file).unwrap();
        // self.current_duration = Some(source.total_duration().unwrap());
        // dbg!(self.current_duration);

        self.sink.append(source);
    }

    pub fn pause(&mut self) {
        self.sink.pause();
    }

    pub fn resume(&mut self) {
        self.sink.play();
    }

    pub fn play(&mut self) {
        self.sink.play();
    }
}

