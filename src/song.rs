use rodio::{OutputStreamHandle, OutputStream, Decoder, Sink};
use std::sync::atomic::Ordering::Relaxed;

pub struct Song {
    // N.B. KEEP STREAM HANDLE HERE TO NOT DROP IT!
    // this is important for playing audio.
    _stream_handle: OutputStreamHandle,
    _stream: OutputStream,
    pub sink: Sink,

    pub total_duration: Option<std::time::Duration>,
}

impl Song {
    pub fn new() -> Song {
        let (_stream, _stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&_stream_handle).unwrap();
        let mut s = Song {
            _stream_handle,
            _stream,
            sink,
            total_duration: None,
        };

        s.append_song(0);
        s
    }

    /// changes the currently playing song based on crate::SONG_INDEX
    /// you needn't worry about synchronisation or whatnot.
    pub fn rejitter_song(&mut self) {
        self.sink.stop();
        let song = crate::SONG_INDEX.load(Relaxed);
        self.append_song(song);
        self.play();
    }

    fn append_song(&mut self, index: usize) {
        use std::{fs::File, io::BufReader};
        use rodio::Source;

        let to_open = &crate::PLAYLIST.read().unwrap();
        if index >= to_open.len() {
            // we've overflowed. callers account for this, so return immedately.
            return;
        }
        let file = BufReader::new(File::open(&to_open[index]).unwrap());
        let source = Decoder::new(file).unwrap();
        self.total_duration = source.total_duration();

        self.sink.append(source);
    }

    pub fn pause(&mut self) {
        self.sink.pause();
    }

    pub fn play(&mut self) {
        self.sink.play();
    }
}

