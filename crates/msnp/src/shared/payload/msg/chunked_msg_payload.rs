use std::str::from_utf8;
use std::vec::Drain;
use anyhow::anyhow;
use bytes::{Bytes, BytesMut};
use log::debug;
use crate::msnp::error::PayloadError;
use crate::msnp::switchboard::command::msg::MsgPayload;
use crate::shared::models::uuid::Uuid;
use crate::shared::payload::msg::raw_msg_payload::{MsgContentType, RawMsgPayload};
use crate::shared::traits::{TryFromRawMsgPayload, TryFromBytes, IntoBytes};


pub enum ChunkMetadata {
    First {
        message_id: String,
        chunks: u32
    },
    Chunk {
        message_id: String,
        chunk: u32
    }
}

impl ChunkMetadata {
    pub fn chunk(&self) -> u32 {
        match self {
            ChunkMetadata::First { .. } => { 1 }
            ChunkMetadata::Chunk { message_id, chunk } => { *chunk }
        }
    }
}

pub struct ChunkedMsgPayload {
    pub raw_msg_payload: RawMsgPayload,
    pub metadata: ChunkMetadata,
}

impl ChunkedMsgPayload {
    pub fn new(raw_msg_payload: RawMsgPayload, metadata: ChunkMetadata) -> Self {
        Self { raw_msg_payload, metadata }
    }

    pub fn body(&self) -> &[u8] {
        self.raw_msg_payload.body.as_ref()
    }

    pub fn body_owned(&self) -> Bytes {
        self.raw_msg_payload.body.clone()
    }

    pub fn metadata(&self) -> &ChunkMetadata {
        &self.metadata
    }

    pub fn message_id(&self) -> String {
        match &self.metadata {
            ChunkMetadata::First { message_id, chunks } => {
                message_id.clone()
            }
            ChunkMetadata::Chunk { message_id, chunk } => {
                message_id.clone()
            }
        }
    }

    pub fn chunk(&self) -> u32 {
        self.metadata.chunk()
    }

    pub fn is_first(&self) -> bool {
        matches!(self.metadata, ChunkMetadata::First { .. })
    }
}

impl Into<RawMsgPayload> for ChunkedMsgPayload {
    fn into(self) -> RawMsgPayload {
        let mut out = self.raw_msg_payload;

        match self.metadata {
            ChunkMetadata::First { message_id, chunks } => {
                out.add_header_owned("Message-ID".to_string(), message_id);
                out.add_header_owned("Chunks".to_string(), chunks.to_string());
            }
            ChunkMetadata::Chunk { message_id, chunk } => {
                out.add_header_owned("Message-ID".to_string(), message_id);
                out.add_header_owned("Chunk".to_string(), chunk.to_string());
            }
        }

        out
    }
}

impl TryFromRawMsgPayload for ChunkedMsgPayload {
    type Err = PayloadError;

    fn try_from_raw(mut raw: RawMsgPayload) -> Result<Self, Self::Err>
    where
        Self: Sized
    {
        let chunked_metadata = if let Some(chunks) = raw.remove_header("Chunks") {
            let message_id = raw.remove_header("Message-ID").ok_or_else(|| anyhow!("Missing Message-ID header"))?;
            let chunks = chunks.parse::<u32>().map_err(|_| anyhow!("Invalid chunks header value"))?;

            ChunkMetadata::First {
                message_id,
                chunks
            }

        } else if let Some(chunk) = raw.remove_header("Chunk") {
            let chunk = chunk.parse::<u32>().map_err(|_| anyhow!("Invalid chunk header value"))?;
            let message_id = raw.remove_header("Message-ID").ok_or_else(|| anyhow!("Missing Message-ID header"))?;
                ChunkMetadata::Chunk {
                    message_id,
                    chunk
                }
        } else {
            return Err(PayloadError::MandatoryPartNotFound { name: "Chunk or Chunks".to_string(), payload: "".to_string() });
        };

        Ok(Self {
            raw_msg_payload: raw,
            metadata: chunked_metadata
        })
    }

}

impl IntoBytes for ChunkedMsgPayload {
    fn into_bytes(self) -> Vec<u8> {
        let raw: RawMsgPayload = self.into();
        raw.into_bytes()
    }
}

pub struct MsgChunks {
    chunks: Vec<ChunkedMsgPayload>,
}

impl MsgChunks {

    pub fn from_first_chunk(first: ChunkedMsgPayload) -> Result<Self, PayloadError> {

        if let ChunkMetadata::First {message_id, chunks } = &first.metadata {
            let mut chunks_vec = Vec::new();
            chunks_vec.push(first);
            Ok(Self {
                chunks: chunks_vec,
            })
        } else {
            Err(PayloadError::MandatoryPartNotFound { name: "First Chunk".to_string(), payload: "MSG".to_string() })
        }
    }

    pub fn split_into_chunks(mut raw_msg_payload: RawMsgPayload, chunk_size: usize) -> MsgChunks {
        let msg_id = Uuid::new();

        let total_chunks = raw_msg_payload.body.len().div_ceil(chunk_size);
        let mut out = MsgChunks {
            chunks: Vec::with_capacity(total_chunks),
        };

        let body: Bytes = raw_msg_payload.body.clone();

        let mut start = 0;
        for (index, _) in (0..total_chunks).enumerate() {
            let end = std::cmp::min(start + chunk_size, body.len());

            let chunk_body = body.slice(start..end);
            start = end;

            let chunked_msg_payload = if index == 0 {

                let metadata = ChunkMetadata::First {
                    message_id: msg_id.to_string(),
                    chunks: total_chunks as u32,
                };

                //Copy first chunk to steal all of it's headers
                let mut first_chunk_raw_msg = raw_msg_payload.clone();
                first_chunk_raw_msg.body = chunk_body;

                ChunkedMsgPayload::new(first_chunk_raw_msg, metadata)
            } else {
                let metadata = ChunkMetadata::Chunk { message_id: msg_id.to_string(), chunk: index as u32 };
                let mut chunk_raw_msg = raw_msg_payload.clone();
                chunk_raw_msg.body = chunk_body;
                ChunkedMsgPayload::new(chunk_raw_msg, metadata)
            };

            out.append_chunk(chunked_msg_payload);
        }

        out
    }

    pub fn append_chunk(&mut self, chunk: ChunkedMsgPayload) {
        self.chunks.push(chunk);
    }

    pub fn first(&self) -> &ChunkedMsgPayload {
        &self.chunks[0]
    }

    fn take_first(&mut self) -> ChunkedMsgPayload {
        self.chunks.remove(0)
    }

    pub fn get_chunk_count(&self) -> Result<u32, PayloadError> {
        if let ChunkMetadata::First { message_id, chunks } = &self.first().metadata {
            Ok(*chunks)
        } else {
            Err(PayloadError::MandatoryPartNotFound { name: "Chunks".to_string(), payload: "MSG".to_string() })
        }
    }

    pub fn is_complete(&self) -> Result<bool, PayloadError> {
        debug!("is_complete: {} == {}", self.chunks.len() as u32, self.get_chunk_count()?);
        Ok(self.chunks.len() as u32 == self.get_chunk_count()?)
    }

    pub fn chunks_mut(&mut self) -> &mut Vec<ChunkedMsgPayload> {
        &mut self.chunks
    }

    pub fn merge_chunks(mut self) -> Result<MsgPayload, PayloadError> {
        let total_chunks = self.get_chunk_count()?;
        let mut first: RawMsgPayload = self.take_first().into();
        let _ = first.headers.remove("Chunks").ok_or_else(|| PayloadError::MandatoryPartNotFound { name: "Chunks".to_string(), payload: "MSG Payload".to_string() })?;

        let mut reassembled_body = BytesMut::from(&first.body[..]);

        for current_chunk in 1..total_chunks {
            match self.chunks.iter().position(|chunk| chunk.chunk() == current_chunk) {
                None => {
                    return Err(PayloadError::MandatoryPartNotFound { name: format!("Chunk {}", current_chunk), payload: "MSG".to_string() });
                }
                Some(found_index) => {
                    let found_chunk = self.chunks.remove(found_index);
                    let mut found_body = found_chunk.body();

                    let test = from_utf8(&found_body).unwrap();
                    let vec: Vec<&str> = test.split("\r\n").collect();

                    reassembled_body.extend_from_slice(&mut found_body)
                }
            }
        }

        first.body = reassembled_body.freeze();

        MsgPayload::try_from_raw(first)
    }

}