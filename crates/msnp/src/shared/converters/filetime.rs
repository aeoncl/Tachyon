use std::convert::Infallible;
use std::fmt::{Display, Formatter};
use std::num::TryFromIntError;
use std::str::FromStr;
use byteorder::{BigEndian, ByteOrder, LittleEndian};
use chrono::{Local, Utc};
use crate::msnp::error::PayloadError;

pub struct FileTime(i64);

impl Display for FileTime {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut buf: [u8; 8] = [0; 8];
        BigEndian::write_u64(&mut buf, self.0 as u64);

        let most_significant_bytes = hex::encode_upper(&buf[0..4]);
        let least_significant_bytes = hex::encode_upper(&buf[4..8]);

        write!(f, "[{}:{}]", least_significant_bytes, most_significant_bytes)
    }
}

impl FromStr for FileTime {
    type Err = Infallible;
    //TODO make a parse error

    fn from_str(filetime: &str) -> Result<Self, Self::Err> {
        let split: Vec<&str> = filetime.trim_start_matches("[").trim_end_matches("]").split(":").collect();
        let lsb = split.get(0).unwrap_or(&"00000000");
        let msb = split.get(1).unwrap_or(&"00000000");

        let lsb_decoded = hex::decode(lsb.as_bytes()).unwrap();
        let msb_decoded = hex::decode(msb.as_bytes()).unwrap();

        let mut buf: Vec<u8> = Vec::new();
        buf.extend_from_slice(&lsb_decoded);
        buf.extend_from_slice(&msb_decoded);

        return Ok(Self(LittleEndian::read_u64(&buf) as i64));
    }
}

impl FileTime {
    pub fn from_local_datetime(datetime: chrono::DateTime<Local>) -> Self {
        Self::from_utc_datetime(datetime.to_utc())
    }

    pub fn from_utc_datetime(datetime: chrono::DateTime<Utc>) -> Self {
        let ft_dt = filetime_type::FileTime::from_datetime(datetime);
        let ft_dt_i64 = ft_dt.filetime();
        Self(ft_dt_i64)
    }
}

fn datetime_to_win32_filetime(datetime: &chrono::DateTime<Utc>) -> Result<String, TryFromIntError> {
    //https://devblogs.microsoft.com/oldnewthing/20090306-00/?p=18913
    // Diff in milliseconds between Linux Epoch (1970) and Win32 epoch (January 1, 1601).
    const EPOCH_DIFF_IN_MS: i64 = 11644473600000;
    const MS_TO_100NS: i64 = 10000;

    //https://learn.microsoft.com/fr-fr/windows/win32/api/minwinbase/ns-minwinbase-filetime
    let ts_in_100_nanosec_interval = u64::try_from((EPOCH_DIFF_IN_MS + datetime.timestamp_millis()) * MS_TO_100NS)?;

    let mut buf: [u8; 8] = [0; 8];
    BigEndian::write_u64(&mut buf, ts_in_100_nanosec_interval);

    let most_significant_bytes = hex::encode_upper(&buf[0..4]);
    let least_significant_bytes = hex::encode_upper(&buf[4..8]);
    Ok(format!("[{}:{}]", least_significant_bytes, most_significant_bytes))

}