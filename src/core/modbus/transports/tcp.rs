use crate::prelude::data_types::StandardDecoder;
use crate::prelude::*;
use enum_primitive::FromPrimitive;
use futures_util::sink::SinkExt;
use log::{debug, error, trace};
use std::collections::HashSet;
use std::io::Write;
use std::net::SocketAddr;
use std::sync::Arc;

use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;
use tokio::sync::{Mutex, Notify};
use tokio_stream::StreamExt;
use tokio_util::bytes::{Buf, BufMut, BytesMut};
use tokio_util::codec::{Decoder, Encoder, FramedRead, FramedWrite};

use crate::prelude::Decoder as LocalDecoder;
use crate::queue::buffer::Topic;

pub const MODBUS_PROTOCOL_TCP: u16 = 0x0000;

/// The default port by which ModBus communication
/// occurs over a standard connection to a LabJack device over ethernet.
///
/// Note: The [`Discover`] module will use a different port since it operates over the UDP broadcast methodology.
pub const MODBUS_TCP_DEFAULT_PORT: u16 = 502;

/// Describes the maximum ammount of data
/// that can be sent in an Ethernet packet.
///
/// Note that the Wi-Fi maximum-size is different,
/// however this library does not cater to that
/// use-case.
///
/// Referenced from the [Packet Size Limits](https://support.labjack.com/docs/protocol-details-direct-modbus-tcp#ProtocolDetails[DirectModbusTCP]-PacketSizeLimits) documentation.
pub const MAX_DATA_LENGTH: usize = 1040;

/// As referenced in the LabJack manual fields documentation for ModBus messages,
/// the UnitID field is not used (as bridging is not used). Therefore, the default
/// value is suggested to be the u8 literal, 1. Alternatively, `0b00000001`.
///
/// Referenced Documentation: [LabJack Modbus Protocol Details: Fields](https://support.labjack.com/docs/protocol-details-direct-modbus-tcp#ProtocolDetails[DirectModbusTCP]-Fields).
const BASE_UNIT_ID: u8 = 1;

/// The base transaction ID. We use this value to identify a unique transaction,
/// such that the LabJack will relay this value back to us.
///
/// We have [`u16::MAX`] (or 65535) values. The use of values is implementation-dependent
/// but often uses a cycle to perform a checked addition to the existing transaction id
/// for each new message.
///
/// Referenced Documentation: [LabJack Modbus Protocol Details: Fields](https://support.labjack.com/docs/protocol-details-direct-modbus-tcp#ProtocolDetails[DirectModbusTCP]-Fields).
const STARTING_TRANSACTION_ID: u16 = 0;

// TODO: Redo the responsibilities of the transaction id here...

#[derive(Debug)]
pub struct TcpTransport {
    transaction_id: u16,
    unit_id: u8,

    cancel: Arc<Notify>,
    stream_write: Arc<Mutex<FramedWrite<OwnedWriteHalf, BytesCodec>>>,
    topic: Arc<Topic>,

    /// A hashset of existing transactions to indicate which values
    /// the transaction_id can take. When it's length is equal to
    /// [`u16::MAX`], no more transactions can be made. It is key
    /// that upon the completion of a transaction, it's identifier
    /// is removed from this set.
    existing_transactions: HashSet<u16>,
}

impl TcpTransport {
    pub fn new(stream: TcpStream) -> TcpTransport {
        let (read, write) = stream.into_split();
        let fr = FramedRead::new(read, BytesCodec);
        let fw = FramedWrite::new(write, BytesCodec);

        let topic = Topic::new();
        let notify = Arc::new(Notify::new());

        let listener_topic = Arc::clone(&topic);
        let listener_notify = Arc::clone(&notify);

        tokio::spawn(async move {
            TcpTransport::listen(
                Arc::clone(&listener_topic),
                Arc::clone(&listener_notify),
                fr,
            )
            .await
        });

        TcpTransport {
            unit_id: BASE_UNIT_ID,
            transaction_id: STARTING_TRANSACTION_ID,

            cancel: notify,
            stream_write: Arc::new(Mutex::new(fw)),

            topic: Arc::clone(&topic),
            existing_transactions: HashSet::new(),
        }
    }

    async fn listen(
        topic: Arc<Topic>,
        notify: Arc<Notify>,
        mut read: FramedRead<OwnedReadHalf, BytesCodec>,
    ) {
        loop {
            tokio::select! {
                data = read.next() => {
                    match data {
                        Some(Ok((header, packet))) => {
                            trace!(
                                "Obtained packet of size {}. TxnID={}",
                                header.length,
                                header.transaction_id
                            );

                            // Publish the packet through to the subscriber
                            topic.publish(header, packet).await;
                        }
                        Some(Err(err)) => {
                            error!("Error reading from `BytesCodec` stream: {:?}", err);
                        }
                        _ => {}
                    }
                }
                _ = notify.notified() => {
                    break
                }
            }
        }

        debug!("Listening ended, cancellation notice issued.")
    }

    fn compositor(&mut self) -> Compositor {
        Compositor {
            transaction_id: &mut self.transaction_id,
            unit_id: self.unit_id,
        }
    }

    fn validate_response_header(req: &Header, resp: &Header) -> Result<(), Error> {
        if req.transaction_id != resp.transaction_id || resp.protocol_id != MODBUS_PROTOCOL_TCP {
            Err(Error::InvalidResponse)
        } else {
            Ok(())
        }
    }

    fn validate_response_code(req: &[u8], res: &[u8]) -> Result<(), Error> {
        let req_code = *req.get(7).ok_or(Error::InvalidResponse)?;
        let res_code = *res.get(7).ok_or(Error::InvalidResponse)?;

        match res_code {
            code if code == req_code + 0x80 => {
                let exception = *res.get(8).ok_or(Error::InvalidResponse)?;
                match ExceptionCode::from_u8(exception) {
                    Some(code) => Err(Error::Exception(code)),
                    None => Err(Error::InvalidResponse),
                }
            }
            code if code == req_code => Ok(()),
            _ => Err(Error::InvalidResponse),
        }
    }

    fn get_reply_data(reply: &[u8], expected_bytes: usize) -> Result<&[u8], Error> {
        let given_response_length = *reply
            .get(8)
            .ok_or(Error::InvalidData(Reason::UnexpectedReplySize))?
            as usize;
        let reply_length_does_not_match = reply.len() != MODBUS_HEADER_SIZE + expected_bytes + 2;

        if given_response_length != expected_bytes || reply_length_does_not_match {
            return Err(Error::InvalidData(Reason::UnexpectedReplySize));
        }

        let reply_data = reply
            .get(MODBUS_HEADER_SIZE + 2..)
            .ok_or(Error::InvalidData(Reason::UnexpectedReplySize))?;

        Ok(reply_data)
    }
}

impl Transport for TcpTransport {
    type Error = Error;

    async fn write(&mut self, function: WriteFunction) -> Result<(), Self::Error> {
        let ComposedMessage { content, .. } = self.compositor().compose_write(&function)?;

        self.stream_write.lock().await.send(content.clone()).await?;

        let (header, packet) = self.topic.wait_on(self.transaction_id).await?;
        let response_header = Header::unpack(packet.as_slice())?;

        TcpTransport::validate_response_header(&header, &response_header)?;
        TcpTransport::validate_response_code(&content, packet.as_slice())
    }

    async fn read(&mut self, function: ReadFunction) -> Result<LabJackDataValue, Self::Error> {
        let ComposedMessage {
            content,
            header,
            expected_bytes,
        } = self.compositor().compose_read(&function)?;

        // self.stream_write.lock().await.
        self.stream_write.lock().await.send(content.clone()).await?;

        // We make a copy of the TID so it is not modified whilst in use
        let (response_header, packet) = self.topic.wait_on(self.transaction_id).await?;
        debug!("Response contains ... Header={response_header:?}. Packet={packet:?}");

        TcpTransport::validate_response_header(&header, &response_header)?;
        TcpTransport::validate_response_code(&content, &packet)?;

        let bytes = TcpTransport::get_reply_data(&packet, expected_bytes)?;
        debug!("Expected reply data: {bytes:?}");

        // TODO: Check expected length and remove 1.. offset.
        StandardDecoder { bytes }.decode_as(function.0.data_type)
    }

    // fn feedback(&mut self, data: &[FeedbackFunction]) -> Result<Box<[u8]>, Self::Error> {
    //     let ComposedMessage {
    //         content,
    //         header,
    //         expected_bytes,
    //     } = self.compositor().compose_feedback(data)?;
    //     let mut reply = vec![0; MODBUS_HEADER_SIZE + expected_bytes + 2].into_boxed_slice();
    //
    //     self.stream.write_all(&content).map_err(Error::Io)?;
    //     self.stream.read(&mut reply).map_err(Error::Io)?;
    //
    //     let reply_header_raw = &reply
    //         .get(..MODBUS_HEADER_SIZE)
    //         .ok_or(Error::InvalidResponse)?;
    //     let resp_hd = Header::unpack(reply_header_raw)?;
    //
    //     TcpTransport::validate_response_header(&header, &resp_hd)?;
    //     TcpTransport::validate_response_code(&content, &reply)?;
    //     TcpTransport::get_reply_data(&reply, expected_bytes).map(Box::from)
    // }
}

/// The TCP ModBus client.
///
/// Example:
/// ```
/// // Import prelude items
/// use labjack::prelude::*;
///
/// // Connect to our LabJack over TCP
/// let mut device = LabJack::connect::<Emulated>(-2).expect("Must connect");
/// // Read the AIN55 pin without an extended feature
/// let voltage = device.read_register(*AIN55).expect("Must read");
///
/// assert!(matches!(voltage, LabJackDataValue::Float32(..)), "had {voltage:?}");
/// println!("Voltage(as f64)={}", voltage.as_f64());
/// ```
pub struct Tcp;

impl Connect for Tcp {
    type Transport = TcpTransport;

    async fn connect(device: LabJackDevice) -> Result<Self::Transport, Error> {
        let addr = SocketAddr::new(device.ip_address, MODBUS_COMMUNICATION_PORT);
        let stream = TcpStream::connect(addr).await.map_err(Error::Io)?;

        Ok(TcpTransport::new(stream))
    }
}

#[derive(Debug)]
struct BytesCodec;

impl Decoder for BytesCodec {
    type Item = (Header, Vec<u8>);
    type Error = Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() < MODBUS_HEADER_SIZE {
            // Not enough data to read length marker.
            return Ok(None);
        }

        // Read header prefix
        let mut length_bytes = [0u8; MODBUS_HEADER_SIZE];
        length_bytes.copy_from_slice(
            src.get(..MODBUS_HEADER_SIZE)
                .ok_or(Error::InvalidData(Reason::UnexpectedReplySize))?,
        );
        let header = Header::unpack(length_bytes.as_ref())?;

        // Check that the length is not too large to avoid a DoS
        if header.length > MAX_DATA_LENGTH as u16 {
            return Err(Error::Queue(QueueError::FrameSizeTooLarge));
        }

        // We include the UnitID as a part of the header, therefore we must subtract its length
        // from the size of the expected message.
        let expected_size = MODBUS_HEADER_SIZE + header.length as usize - 1;
        if src.len() < expected_size {
            // We reserve the expected space
            src.reserve(expected_size - src.len());

            // We inform the Framed that we need more bytes to form the next
            // frame.
            return Ok(None);
        }

        // Use advance to modify src such that it no longer contains this frame.
        let data = src
            .get(..expected_size)
            .ok_or(Error::InvalidResponse)?
            .to_vec();
        src.advance(expected_size);

        // Return the packet as bytes
        Ok(Some((header, data)))
    }
}

impl Encoder<Vec<u8>> for BytesCodec {
    type Error = Error;

    fn encode(&mut self, item: Vec<u8>, dst: &mut BytesMut) -> Result<(), Self::Error> {
        dst.writer().write_all(item.as_slice()).map_err(Error::Io)
    }
}

#[cfg(test)]
mod test {
    use log::debug;
    use std::time::Duration;
    use tokio::io::AsyncWriteExt;
    use tokio::join;
    use tokio::net::{TcpListener, TcpStream};
    use tokio::time::sleep;

    use crate::core::{LabJackDataValue, ReadFunction};
    use crate::prelude::{TcpTransport, Transport, TEST_UINT32};

    async fn setup() -> (TcpTransport, TcpStream) {
        env_logger::init();

        debug!("Testing validate waterfall");

        // Bind to any usable port
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("Must bind to a port");
        let addr = listener.local_addr().unwrap();

        debug!("Listening on {}", addr);

        let reader = TcpStream::connect(addr).await.unwrap();
        let mut transport = TcpTransport::new(reader);

        debug!("Both transports connected");

        let (mut reader, ..) = listener.accept().await.expect("Must accept connection");
        debug!("Accepted Connection");

        (transport, reader)
    }

    #[tokio::test]
    async fn validate_waterfall() {
        let (mut transport, mut writer) = setup().await;

        let join = tokio::spawn(async move {
            sleep(Duration::from_millis(100)).await;
            writer
                .write(&[
                    0x00, 0x01, 0x00, 0x00, 0x00, 0x07, 0x01, 0x03, 0x04, 0x00, 0x11, 0x22, 0x33,
                ])
                .await
                .expect("Must write");
            debug!("Written value to other stream");
        });

        let join2 = tokio::spawn(async move {
            let value = transport
                .read(ReadFunction(*TEST_UINT32))
                .await
                .expect("Must write read fn.");
            assert_eq!(value, LabJackDataValue::Uint32(0x00112233));
            transport.cancel.notify_one();
        });

        join!(join2, join);
    }

    #[tokio::test]
    async fn validate_async_return() {
        let (mut transport, mut writer) = setup().await;

        let join = tokio::spawn(async move {
            sleep(Duration::from_millis(100)).await;

            writer
                .write(&[
                    0x00, 0x01, 0x00, 0x00, 0x00, 0x07, 0x01, 0x03, 0x04, 0x00, 0x11, 0x22, 0x44,
                ])
                .await
                .expect("Must write");

            sleep(Duration::from_millis(100)).await;

            writer
                .write(&[
                    0x00, 0x02, 0x00, 0x00, 0x00, 0x07, 0x01, 0x03, 0x04, 0x00, 0x11, 0x22, 0x33,
                ])
                .await
                .expect("Must write");

            sleep(Duration::from_millis(100)).await;

            writer
                .write(&[
                    0x00, 0x03, 0x00, 0x00, 0x00, 0x07, 0x01, 0x03, 0x04, 0x00, 0x11, 0x22, 0x22,
                ])
                .await
                .expect("Must write");

            sleep(Duration::from_millis(100)).await;

            debug!("Written value to other stream");
        });

        let join2 = tokio::spawn(async move {
            let value = transport
                .read(ReadFunction(*TEST_UINT32))
                .await
                .expect("Must write read fn.");
            assert_eq!(value, LabJackDataValue::Uint32(0x00112244));

            let value = transport
                .read(ReadFunction(*TEST_UINT32))
                .await
                .expect("Must write read fn.");
            assert_eq!(value, LabJackDataValue::Uint32(0x00112233));

            let value = transport
                .read(ReadFunction(*TEST_UINT32))
                .await
                .expect("Must write read fn.");
            assert_eq!(value, LabJackDataValue::Uint32(0x00112222));

            transport.cancel.notify_one();
        });

        join!(join2, join);
    }
}
