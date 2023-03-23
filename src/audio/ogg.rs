use crc::Algorithm;

// CRC algorithm for OGG
const CRC_32_OGG: Algorithm<u32> = Algorithm {
    width: 32,
    poly: 0x04C11DB7,
    init: 0x00000000,
    refin: false,
    refout: false,
    xorout: 0x00000000,
    check: 0x00000000,
    residue: 0x00000000,
};

#[derive(Clone)]
pub struct OggHeader {
    pub magic: [u8; 4],
    pub version: u8,
    pub header_type: u8,
    pub granule_position: u64,
    pub stream_serial_number: u32,
    pub page_sequence_number: u32,
    pub checksum: u32,
    pub page_segments: u8,
    pub segment_table: Vec<u8>,
}

impl OggHeader {
    pub fn new(
        header_type: u8,
        granule_position: u64,
        stream_serial_number: u32,
        page_sequence_number: u32,
        page_segments: u8,
        segment_table: Vec<u8>,
    ) -> Self {
        Self {
            magic: [0x4F, 0x67, 0x67, 0x53], // OggS
            version: 0,
            header_type,
            granule_position,
            stream_serial_number,
            page_sequence_number,
            checksum: 0,
            page_segments,
            segment_table,
        }
    }
}

impl Into<Vec<u8>> for OggHeader {
    fn into(self) -> Vec<u8> {
        let mut data = Vec::new();

        data.extend_from_slice(&self.magic);
        data.push(self.version);
        data.push(self.header_type);
        data.extend_from_slice(&self.granule_position.to_le_bytes());
        data.extend_from_slice(&self.stream_serial_number.to_le_bytes());
        data.extend_from_slice(&self.page_sequence_number.to_le_bytes());
        data.extend_from_slice(&self.checksum.to_le_bytes());
        data.push(self.page_segments);
        data.extend_from_slice(&self.segment_table);

        data
    }
}

pub struct OggBody {
    pub data: Vec<u8>,
}

impl OggBody {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }
}

impl Into<Vec<u8>> for OggBody {
    fn into(self) -> Vec<u8> {
        self.data
    }
}

pub struct OggPage {
    pub header: OggHeader,
    pub body: OggBody,
}

impl OggPage {
    pub fn new(
        header_type: u8,
        granule_position: u64,
        stream_serial_number: u32,
        page_sequence_number: u32,
        page_segments: u8,
        segment_table: Vec<u8>,
        body: Vec<u8>,
    ) -> Self {
        let mut header = OggHeader::new(
            header_type,
            granule_position,
            stream_serial_number,
            page_sequence_number,
            page_segments,
            segment_table,
        );

        let mut data = Vec::new();
        data.extend_from_slice(&Into::<Vec<u8>>::into(header.clone()));
        data.extend_from_slice(&body);

        // Checksum is calculated with the checksum field set to 0 with the data
        let checksum = crc::Crc::<u32>::new(&CRC_32_OGG).checksum(&data);

        header.checksum = checksum;

        Self {
            header,
            body: OggBody::new(body),
        }
    }
}

impl Into<Vec<u8>> for OggPage {
    fn into(self) -> Vec<u8> {
        let mut data = Vec::new();

        data.extend_from_slice(&Into::<Vec<u8>>::into(self.header));
        data.extend_from_slice(&Into::<Vec<u8>>::into(self.body));

        data
    }
}

pub fn get_segment_table(data: &[u8], segment_size: usize) -> Vec<u8> {
    let mut segment_table = Vec::new();

    for chunk in data.chunks(segment_size) {
        for segment in chunk.chunks(255) {
            segment_table.push(segment.len() as u8);
        }
    }

    segment_table
}
