use axum::{
    body::Bytes,
    extract::{ContentLengthLimit, Multipart},
    BoxError,
};
use std::io::{stdout, Write};
use itertools::Itertools;
use hound;

use clap::{App, Arg};
use futures::{Stream, TryStreamExt};
use std::io;
use tokio::{fs::File, io::BufWriter};
use tokio_util::io::StreamReader;

use cheetah::CheetahBuilder;

pub async fn save(
    ContentLengthLimit(mut multipart): ContentLengthLimit<Multipart, { 2500 * 1024 * 1024 }>,
) -> String {
    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap().to_string();
        let file_name = field.file_name().unwrap().to_string();
        let content_type = field.content_type().unwrap().to_string();
        println!("uploading `{}`, `{}`, `{}`", name, file_name, content_type);
        let res = stream_to_file(field).await;
        dbg!("{}", &res);
    }

    let matches = App::new("Picovoice Cheetah Rust File Demo")
    .arg(
        Arg::with_name("model_path")
            .long("model_path")
            .value_name("PATH")
            .help("Path to the file containing model parameter.")
            .takes_value(true),
    )
    .get_matches();
    let model_path = matches.value_of("model_path");

    let access_key = "nhdZq87rVCiTpYkm2tIVPy6r71TC3Vrk+OkL+iAnhIu+svJzEismGQ==";
    
    let mut cheetah_builder = CheetahBuilder::new(access_key);

    if let Some(model_path) = model_path {
        cheetah_builder.model_path(model_path);
    }

    let cheetah = cheetah_builder.init().expect("Failed to create Cheetah");

    let mut wav_reader = match hound::WavReader::open("test2.wav") {
        Ok(reader) => reader,
        Err(err) => panic!(
            "Failed to open .wav audio file {}: {}",
            "test2.wav",
            err
        ),
    };

    if wav_reader.spec().sample_rate != cheetah.sample_rate() {
        // panic!(
        //     "Audio file should have the expected sample rate of {}, got {}",
        //     cheetah.sample_rate(),
        //     wav_reader.spec().sample_rate
        // );

        return "Audio file should have the expected sample rate of 16000".to_string();
    }

    if wav_reader.spec().channels != 1u16 {
        // panic!(
        //     "Audio file should have the expected number of channels 1, got {}",
        //     wav_reader.spec().channels
        // );

        return "Audio file should have the expected number of channels 1".to_string();
    }

    if wav_reader.spec().bits_per_sample != 16u16
        || wav_reader.spec().sample_format != hound::SampleFormat::Int
    {
        return "WAV format should be in the signed 16 bit format".to_string();
        //panic!("WAV format should be in the signed 16 bit format",);
    }

    let mut owned_string: String = "".to_owned();
    for frame in &wav_reader.samples().chunks(cheetah.frame_length() as usize) {
        let frame: Vec<i16> = frame.map(|s| s.unwrap()).collect_vec();

        if frame.len() == cheetah.frame_length() as usize {
            let partial_transcript = cheetah.process(&frame).unwrap();
            print!("{}", partial_transcript.transcript);
            stdout().flush().expect("Failed to flush");

            let another_owned_string: String = String::from(partial_transcript.transcript);
            owned_string.push_str(&another_owned_string);
        }
    }

    let final_transcript = cheetah.flush().unwrap();
    println!("{}", final_transcript.transcript);
    let another_owned_string: String = String::from(final_transcript.transcript);
    owned_string.push_str(&another_owned_string);

    return owned_string;
}

// Save a `Stream` to a file
async fn stream_to_file<S, E>(stream: S) -> Result<(), io::Error>
where
    S: Stream<Item = Result<Bytes, E>>,
    E: Into<BoxError>,
{
    // Convert the stream into an `AsyncRead`.
    let body_with_io_error = stream.map_err(|err| io::Error::new(io::ErrorKind::Other, err));
    let body_reader = StreamReader::new(body_with_io_error);
    futures::pin_mut!(body_reader);

    // Create the file. `File` implements `AsyncWrite`.
    let mut file = BufWriter::new(File::create("test2.wav").await?);

    // Copy the body into the file.
    tokio::io::copy(&mut body_reader, &mut file).await?;

    Ok(())
}
