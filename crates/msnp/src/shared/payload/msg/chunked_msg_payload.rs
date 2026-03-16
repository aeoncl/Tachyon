use std::str::from_utf8;
use anyhow::anyhow;
use log::debug;
use crate::msnp::error::PayloadError;
use crate::msnp::switchboard::command::msg::MsgPayload;
use crate::shared::payload::msg::raw_msg_payload::{MsgContentType, RawMsgPayload};
use crate::shared::traits::{TryFromRawMsgPayload, TryFromBytes, IntoBytes};


pub enum ChunkMetadata {
    First {
        mime_version: String,
        content_type: MsgContentType,
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
    pub body: Vec<u8>,
    pub metadata: ChunkMetadata,
}

impl ChunkedMsgPayload {
    pub fn new(body: Vec<u8>, metadata: ChunkMetadata) -> Self {
        Self { body, metadata }
    }

    pub fn body(&self) -> &Vec<u8> {
        &self.body
    }

    pub fn metadata(&self) -> &ChunkMetadata {
        &self.metadata
    }

    pub fn message_id(&self) -> String {
        match &self.metadata {
            ChunkMetadata::First { mime_version, content_type, message_id, chunks } => {
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
        let mut out = RawMsgPayload::default();
        out.body = self.body;

        match self.metadata {
            ChunkMetadata::First { mime_version, content_type, message_id, chunks } => {
                out.add_header_owned("MIME-Version".to_string(), mime_version);
                out.add_header_owned("Content-Type".to_string(), content_type.to_string());
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

    fn try_from_raw(raw: RawMsgPayload) -> Result<Self, Self::Err>
    where
        Self: Sized
    {
        let chunked_metadata = if let Some(chunks) = raw.get_header("Chunks") {
            let mime_version = raw.get_header("MIME-Version").ok_or_else(|| anyhow!("Missing MIME-Version header"))?.to_string();
            let content_type = raw.get_content_type().map_err(|e| anyhow!("Failed to get content type: {}", e))?;
            let message_id = raw.get_header("Message-ID").ok_or_else(|| anyhow!("Missing Message-ID header"))?.to_string();
            let chunks = chunks.parse::<u32>().map_err(|_| anyhow!("Invalid chunks header value"))?;

            ChunkMetadata::First {
                mime_version,
                content_type,
                message_id,
                chunks
            }
        } else if let Some(chunk) = raw.get_header("Chunk") {
            let chunk = chunk.parse::<u32>().map_err(|_| anyhow!("Invalid chunk header value"))?;
            let message_id = raw.get_header("Message-ID").ok_or_else(|| anyhow!("Missing Message-ID header"))?.to_string();
                ChunkMetadata::Chunk {
                    message_id,
                    chunk
                }
        } else {
            return Err(PayloadError::MandatoryPartNotFound { name: "Chunk or Chunks".to_string(), payload: "".to_string() });
        };

        Ok(Self {
            body: raw.body,
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

        if let ChunkMetadata::First { mime_version, content_type, message_id, chunks } = &first.metadata {
            let mut chunks_vec = Vec::new();
            chunks_vec.push(first);
            Ok(Self {
                chunks: chunks_vec,
            })
        } else {
            Err(PayloadError::MandatoryPartNotFound { name: "First Chunk".to_string(), payload: "MSG".to_string() })
        }
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
        if let ChunkMetadata::First { mime_version, content_type, message_id, chunks } = &self.first().metadata {
            Ok(*chunks)
        } else {
            Err(PayloadError::MandatoryPartNotFound { name: "Chunks".to_string(), payload: "MSG".to_string() })
        }
    }

    pub fn is_complete(&self) -> Result<bool, PayloadError> {
        debug!("is_complete: {} == {}", self.chunks.len() as u32, self.get_chunk_count()?);
        Ok(self.chunks.len() as u32 == self.get_chunk_count()?)
    }

    pub fn drain_chunks(mut self) -> Result<MsgPayload, PayloadError> {
        let total_chunks = self.get_chunk_count()?;
        let mut first: RawMsgPayload = self.take_first().into();
        let _ = first.headers.remove("Chunks").ok_or_else(|| PayloadError::MandatoryPartNotFound { name: "Chunks".to_string(), payload: "MSG Payload".to_string() })?;

        for current_chunk in 1..total_chunks {
            match self.chunks.iter().position(|chunk| chunk.chunk() == current_chunk) {
                None => {
                    return Err(PayloadError::MandatoryPartNotFound { name: format!("Chunk {}", current_chunk), payload: "MSG".to_string() });
                }
                Some(found_index) => {
                    let mut found_body = self.chunks.remove(found_index).body;

                    let test = from_utf8(&found_body).unwrap();
                    let vec: Vec<&str> = test.split("\r\n").collect();
                    debug!("Chunk {} found with {} lines", current_chunk, vec.len());
                    debug!("{}", &test);
                    debug!("{:?}", &vec);
                    first.body.append(&mut found_body)
                }
            }
        }

        MsgPayload::try_from_raw(first)
    }

}