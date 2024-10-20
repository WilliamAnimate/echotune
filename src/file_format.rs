use std::{io::{BufReader, Read, Seek}, fs::File};
use echotune::FileFormat;

static FILE_HEADERS: &[(&[u8], FileFormat, u64)] = &[
    ( b"OggS", FileFormat::Ogg, 0 ),
    ( b"ID3", FileFormat::Mp3, 0 ),
    ( b"fLaC", FileFormat::Flac, 0 ),
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

