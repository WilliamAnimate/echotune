use rodio::{OutputStreamHandle, OutputStream, Decoder, Sink};

#[allow(unused)]
pub struct Song {
    // pub songs: Vec<String>,
    pub current_song_index: u16, // keep track of which songs are currently playing, for backtracking.
    stream_handle: OutputStreamHandle, // named so you know
    _stream: OutputStream, // we're not using this, but keeping it just in case.
                           // compiler will optimize it out anyways.
                           // hopefully.
    pub sink: Sink,
    pub current_source: Option<Decoder<std::io::BufReader<std::fs::File>>>,

    pub current_duration: Option<std::time::Duration>,
}

#[allow(unused)]
impl Song {
    pub fn new() -> Song {
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();
        Song {
            // songs: Vec::new(),
            current_song_index: 0,
            stream_handle,
            _stream,
            sink,
            current_source: None,
            current_duration: None,
        }
    }

    fn reset(&mut self) {
        self.sink.stop();
    }

    // pub fn set_queue(&mut self, q: Vec<String>) {
    //     self.songs = q;
    // }

    /// this can silently fail if this is the first song.
    pub fn prev_song(&mut self) {
        self.reset();
        let checked = match self.current_song_index.checked_sub(1) {
            Some(k) => k,
            None => return,
        };
        self.append_song(checked);
        self.play();
    }

    pub fn next_song(&mut self) {
        self.reset();
        self.append_song(self.current_song_index + 1);
        self.play()
    }

    fn append_song(&mut self, index: u16) {
        use std::{fs::File, io::BufReader};
        // use rodio::Source;

        // let to_open = &self.songs[index as usize];
        let to_open = &crate::PLAYLIST.read().unwrap().to_vec();
        dbg!(to_open);
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

