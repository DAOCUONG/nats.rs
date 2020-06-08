//! Encode protocol operations to bytes to send to the server.

use std::io;

use futures::prelude::*;

use crate::connect::ConnectInfo;

/// A protocol operation sent by the client.
#[derive(Debug)]
pub(crate) enum ClientOp<'a> {
    /// `CONNECT {["option_name":option_value],...}`
    Connect(ConnectInfo),

    /// `PUB <subject> [reply-to] <#bytes>\r\n[payload]\r\n`
    Pub {
        subject: &'a str,
        reply_to: Option<&'a str>,
        payload: &'a [u8],
    },

    /// `SUB <subject> [queue group] <sid>\r\n`
    Sub {
        subject: &'a str,
        queue_group: Option<&'a str>,
        sid: u64,
    },

    /// `UNSUB <sid> [max_msgs]`
    Unsub { sid: u64, max_msgs: Option<u64> },

    /// `PING`
    Ping,

    /// `PONG`
    Pong,
}

/// Encodes a single operation from the client.
pub(crate) async fn encode(
    mut stream: impl AsyncWrite + Unpin,
    op: ClientOp<'_>,
) -> io::Result<()> {
    dbg!(&op);

    match &op {
        ClientOp::Connect(connect_info) => {
            let op = format!("CONNECT {}\r\n", serde_json::to_string(&connect_info)?);
            stream.write_all(op.as_bytes()).await?;
        }

        ClientOp::Pub {
            subject,
            reply_to,
            payload,
        } => {
            stream.write_all(b"PUB ").await?;
            stream.write_all(subject.as_bytes()).await?;
            stream.write_all(b" ").await?;

            if let Some(reply_to) = reply_to {
                stream.write_all(reply_to.as_bytes()).await?;
                stream.write_all(b" ").await?;
            }

            let mut buf = itoa::Buffer::new();
            stream
                .write_all(buf.format(payload.len()).as_bytes())
                .await?;
            stream.write_all(b"\r\n").await?;

            stream.write_all(payload).await?;
            stream.write_all(b"\r\n").await?;
        }

        ClientOp::Sub {
            subject,
            queue_group,
            sid,
        } => {
            let op = if let Some(queue_group) = queue_group {
                format!("SUB {} {} {}\r\n", subject, queue_group, sid)
            } else {
                format!("SUB {} {}\r\n", subject, sid)
            };
            stream.write_all(op.as_bytes()).await?;
        }

        ClientOp::Unsub { sid, max_msgs } => {
            let op = if let Some(max_msgs) = max_msgs {
                format!("UNSUB {} {}\r\n", sid, max_msgs)
            } else {
                format!("UNSUB {}\r\n", sid)
            };
            stream.write_all(op.as_bytes()).await?;
        }

        ClientOp::Ping => {
            stream.write_all(b"PING\r\n").await?;
        }

        ClientOp::Pong => {
            stream.write_all(b"PONG\r\n").await?;
        }
    }

    Ok(())
}
