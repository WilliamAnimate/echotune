use std::{io::{BufReader, Read, Seek}, fs::File};
use echotune::FileFormat;

static FILE_HEADERS: &[(&[u8], FileFormat, u64)] = &[
    ( b"OggS", FileFormat::Audio, 0 ), // .ogg
    ( b"ID3", FileFormat::Audio, 0 ),  // .mp3
    ( b"fLaC", FileFormat::Audio, 0 ), // .flac
    ( b"RIFF", FileFormat::Audio, 0 ), // .wav
];

/// because `file-format is bloated
/// i did it in 25 SLOC
pub fn check_file(file: &mut BufReader<File>) -> Result<&FileFormat, Box<dyn std::error::Error>> {
    let mut ret: &FileFormat = &FileFormat::Other;
    for (header, format, header_offset) in FILE_HEADERS {
        let mut buf = vec![0; header.len()];
        if let Err(_) = file.seek(std::io::SeekFrom::Start(*header_offset)) {
            // possibly out of bounds
            continue;
        }
        file.read_exact(&mut buf)?;
        if buf != *header {
            continue;
        }

        ret = format;
        break;
    }

    Ok(ret)
}

