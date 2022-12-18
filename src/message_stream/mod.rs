use tokio::{
    io::{AsyncRead, AsyncReadExt},
    sync::mpsc::{self, Sender},
};
use tokio_stream::{wrappers::ReceiverStream, Stream};

use crate::{
    application::{domain::SmlMessages, parser::parse_body},
    transport::SMLMessageBuilder,
};

/// Read SML message stream from a reader
/// ```
/// use std::io::Cursor;
///
/// let cursor = Cursor::new(vec![0x01, 0x02, 0x03]);
/// let message_stream = sml_message_stream(cursor);
/// ```
pub fn sml_message_stream(
    mut stream: impl AsyncRead + Unpin + Send + 'static,
) -> impl Stream<Item = SmlMessages> {
    let (tx, rx) = mpsc::channel::<SmlMessages>(256);

    let mut buf = [0; 512];
    let mut builder = SMLMessageBuilder::Empty;

    tokio::spawn(async move {
        while let Ok(n) = stream.read(&mut buf).await {
            emit_message(&mut builder, &buf[..n], tx.clone()).await;
        }
    });

    ReceiverStream::new(rx)
}

async fn emit_message<'a>(
    builder: &'a mut SMLMessageBuilder,
    buf: &'a [u8],
    tx: Sender<SmlMessages>,
) {
    let mut to_process = buf.to_vec();
    while to_process.len() > 0 {
        builder.record(&to_process);
        to_process = vec![];

        match builder {
            SMLMessageBuilder::Complete { ref data, ref rest } => {
                let result = parse_body(data);
                if let Ok(messages) = result {
                    let _ = tx.send(messages).await;
                }
                if rest.len() == 0 {
                    *builder = SMLMessageBuilder::Empty;
                } else {
                    to_process = rest.to_vec();
                    *builder = SMLMessageBuilder::Empty;
                }
            }
            SMLMessageBuilder::Empty => (),
            SMLMessageBuilder::IncompleteStartSignature(_) => (),
            SMLMessageBuilder::Recording(_) => (),
        }
    }
}
