# The echotune music player usage guide

## Playing a specific file directly

./echotune file.mp3

## Playing a playlist file

./echotune playlist-file

## Managing playlists

echotune does not contain any code to manage or to add to your playlist files. That is up to your text editor.

See [this file](./playlist.md) for more information

As for naming your playlists, you should use your own judgement. For the most part, the name you give your playlist has no effect on runtime.

## Keybindings

> [!INFO]
> This is highly wip and may not actually correlate to the actual keybindings used. refer to `src/input.rs` for actual keybindings.

### Navigating the playlist

k/up arrow   - up one entry
j/down arrow - down one entry

### Misc

<space> - pause or resume

## Supported audio formats

Anything that [rodio](https://github.com/RustAudio/rodio) supports. as of writing, those are the following:

- MPEG Audio Layer III (`.mp3`);
- Vorbis (`.ogg`);
- Waveform Audio File Format (`.wav`);
- Free Lossless Audio Codec (`.FLAC`)

Along with that, some additonal formats may be built into the binary at compile time if you so desire:

- MPEG-4 Part 14 (`.mp4`);
- Advanced Audio Coding (`.aac`)

