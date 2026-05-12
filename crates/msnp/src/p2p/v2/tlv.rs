use byteorder::{BigEndian, ByteOrder};

#[derive(Clone, Debug)]
pub struct TLV {
    pub length: usize,
    pub value_type: u8,
    pub value: Vec<u8>,
}

impl TLV {
    pub fn new(value_type: u8, length: usize, value: Vec<u8>) -> Self {
        TLV {
            length,
            value_type,
            value,
        }
    }

    pub fn empty() -> Self {
        TLV::new(0, 0, Vec::new())
    }

    /// Serializes this single TLV as `[type(1)][length(1)][value(N)]`.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(2 + self.value.len());
        out.push(self.value_type);
        out.push(self.length as u8);
        out.extend_from_slice(&self.value);
        out
    }

    pub fn is_type(&self, value_type: &ValueType) -> bool {
        match value_type {
            ValueType::AckSequenceNumber => self.value_type == 0x02 && self.length == 0x04,
            ValueType::NakSequenceNumber => self.value_type == 0x03 && self.length == 0x04,
            ValueType::ClientPeerInfo => self.value_type == 0x01 && self.length == 0x0c,
            ValueType::SizeOfUntransferData => self.value_type == 0x01 && self.length == 0x08,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ValueType {
    SizeOfUntransferData,
    AckSequenceNumber,
    NakSequenceNumber,
    ClientPeerInfo,
}

/// A wrapper around `Vec<TLV>` that centralises all TLV-related logic:
/// construction from raw bytes, serialisation with 4-byte-aligned padding,
/// and typed lookups.
#[derive(Clone, Debug, Default)]
pub struct TLVList {
    tlvs: Vec<TLV>,
}

impl TLVList {
    // ------------------------------------------------------------------
    // Construction
    // ------------------------------------------------------------------

    /// Creates an empty `TLVList`.
    pub fn new() -> Self {
        TLVList { tlvs: Vec::new() }
    }

    /// Creates a `TLVList` from an existing `Vec<TLV>`.
    pub fn from_tlvs(tlvs: Vec<TLV>) -> Self {
        TLVList { tlvs }
    }

    /// Parses TLVs from a raw byte slice.
    ///
    /// Reads consecutive `[type(1)][length(1)][value(length)]` entries until the
    /// slice is exhausted or a zero-type entry (padding) is encountered.
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let mut offset = 0;
        let mut tlvs = Vec::new();

        while offset < bytes.len() {
            let value_type = match bytes.get(offset) {
                Some(&0) | None => break, // padding or end
                Some(&t) => t,
            };

            let length = match bytes.get(offset + 1) {
                Some(&l) => l as usize,
                None => break,
            };

            let value_start = offset + 2;
            let value_end = value_start + length;

            if value_end > bytes.len() {
                break;
            }

            let value = bytes[value_start..value_end].to_vec();
            tlvs.push(TLV::new(value_type, length, value));
            offset = value_end;
        }

        TLVList { tlvs }
    }

    /// Appends a TLV to the list.
    pub fn push(&mut self, tlv: TLV) {
        self.tlvs.push(tlv);
    }

    /// Removes all TLVs.
    pub fn clear(&mut self) {
        self.tlvs.clear();
    }

    /// Returns the number of TLVs in the list.
    pub fn len(&self) -> usize {
        self.tlvs.len()
    }

    /// Returns `true` if the list contains no TLVs.
    pub fn is_empty(&self) -> bool {
        self.tlvs.is_empty()
    }

    /// Returns an iterator over the TLVs.
    pub fn iter(&self) -> impl Iterator<Item = &TLV> {
        self.tlvs.iter()
    }

    /// Finds the first TLV matching the given [`ValueType`].
    pub fn get_for_type(&self, value_type: &ValueType) -> Option<&TLV> {
        self.tlvs.iter().find(|tlv| tlv.is_type(value_type))
    }

    pub fn get_ack(&self) -> Option<&TLV> {
        self.get_for_type(&ValueType::AckSequenceNumber)
    }

    pub fn get_nak(&self) -> Option<&TLV> {
        self.get_for_type(&ValueType::NakSequenceNumber)
    }

    pub fn get_client_info(&self) -> Option<&TLV> {
        self.get_for_type(&ValueType::ClientPeerInfo)
    }

    pub fn get_untransfered_data_size(&self) -> Option<&TLV> {
        self.get_for_type(&ValueType::SizeOfUntransferData)
    }

    /// Returns the number of remaining (untransferred) bytes advertised by a
    /// `SizeOfUntransferData` TLV, or `0` if no such TLV is present.
    pub fn get_missing_bytes_count(&self) -> u64 {
        self.get_untransfered_data_size()
            .map(|tlv| BigEndian::read_u64(tlv.value.as_slice()))
            .unwrap_or(0)
    }

    /// Returns the total number of bytes occupied by all TLVs *without* padding.
    ///
    /// Each TLV is `2 + value_length` bytes (type + length + value).
    pub fn total_tlv_bytes(&self) -> usize {
        self.tlvs.iter().map(|t| 2 + t.length).sum()
    }

    /// Returns the number of padding bytes required to align the TLV block to a
    /// 4-byte boundary.
    pub fn padding_len(&self) -> usize {
        let total = self.total_tlv_bytes();
        (4 - (total % 4)) % 4
    }

    /// Returns the total serialised size of the TLV block *including* padding.
    pub fn serialized_len(&self) -> usize {
        self.total_tlv_bytes() + self.padding_len()
    }

    /// Serialises all TLVs into a byte vector, with trailing zero bytes to pad
    /// to a 4-byte boundary.
    pub fn to_bytes(&self) -> Vec<u8> {
        let total = self.total_tlv_bytes();
        let padding = self.padding_len();
        let mut out = Vec::with_capacity(total + padding);

        for tlv in &self.tlvs {
            out.extend_from_slice(&tlv.to_bytes());
        }

        out.extend(std::iter::repeat(0u8).take(padding));
        out
    }

    /// Consumes the `TLVList` and returns the inner `Vec<TLV>`.
    pub fn into_inner(self) -> Vec<TLV> {
        self.tlvs
    }
}

impl<'a> IntoIterator for &'a TLVList {
    type Item = &'a TLV;
    type IntoIter = std::slice::Iter<'a, TLV>;

    fn into_iter(self) -> Self::IntoIter {
        self.tlvs.iter()
    }
}

impl std::ops::Index<usize> for TLVList {
    type Output = TLV;

    fn index(&self, index: usize) -> &Self::Output {
        &self.tlvs[index]
    }
}

impl From<Vec<TLV>> for TLVList {
    fn from(tlvs: Vec<TLV>) -> Self {
        TLVList { tlvs }
    }
}

impl From<TLVList> for Vec<TLV> {
    fn from(list: TLVList) -> Self {
        list.tlvs
    }
}
