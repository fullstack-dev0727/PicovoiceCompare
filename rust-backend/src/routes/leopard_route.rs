use axum::{
    body::Bytes,
    extract::{ContentLengthLimit, Multipart},
    BoxError,
};
use futures::{Stream, TryStreamExt};
use std::io;
use tokio::{fs::File, io::BufWriter};
use tokio_util::io::StreamReader;
use leopard::LeopardBuilder;
use leopard::Leopard;

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
    
    let access_key = "nhdZq87rVCiTpYkm2tIVPy6r71TC3Vrk+OkL+iAnhIu+svJzEismGQ==";
    let leopard: Leopard = LeopardBuilder::new(access_key).init().expect("Unable to create Leopard");
    if let Ok(transcript) = leopard.process_file("test1.wav") {
        println!("{}", transcript);
        return String::from(transcript);
    } else {
        return "".to_string();
    }
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
    let mut file = BufWriter::new(File::create("test1.wav").await?);

    // Copy the body into the file.
    tokio::io::copy(&mut body_reader, &mut file).await?;

    Ok(())
}
