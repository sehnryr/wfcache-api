#[derive(Clone)]
pub struct OpusHead {
    pub magic: [u8; 8],
    pub version: u8,
    pub channels: u8,
    pub pre_skip: u16,
    pub input_sample_rate: u32,
    pub output_gain: u16,
    pub channel_mapping_family: u8,
}

impl OpusHead {
    pub fn new(
        version: u8,
        channels: u8,
        pre_skip: u16,
        input_sample_rate: u32,
        output_gain: u16,
        channel_mapping_family: u8,
    ) -> Self {
        Self {
            magic: [0x4F, 0x70, 0x75, 0x73, 0x48, 0x65, 0x61, 0x64], // OpusHead
            version,
            channels,
            pre_skip,
            input_sample_rate,
            output_gain,
            channel_mapping_family,
        }
    }
}

impl Into<Vec<u8>> for OpusHead {
    fn into(self) -> Vec<u8> {
        let mut data = Vec::new();

        data.extend_from_slice(&self.magic);
        data.push(self.version);
        data.push(self.channels);
        data.extend_from_slice(&self.pre_skip.to_le_bytes());
        data.extend_from_slice(&self.input_sample_rate.to_le_bytes());
        data.extend_from_slice(&self.output_gain.to_le_bytes());
        data.push(self.channel_mapping_family);

        data
    }
}

#[derive(Clone)]
pub struct OpusTags {
    pub magic: [u8; 8],
    pub vendor: String,
    pub user_comment_list: Vec<String>,
}

impl OpusTags {
    pub fn new(vendor: String, user_comment_list: Vec<String>) -> Self {
        Self {
            magic: [0x4F, 0x70, 0x75, 0x73, 0x54, 0x61, 0x67, 0x73], // OpusTags
            vendor,
            user_comment_list,
        }
    }
}

impl Into<Vec<u8>> for OpusTags {
    fn into(self) -> Vec<u8> {
        let mut data = Vec::new();

        data.extend_from_slice(&self.magic);

        let vendor = self.vendor.as_bytes();
        data.extend_from_slice(&(vendor.len() as u32).to_le_bytes());
        data.extend_from_slice(vendor);

        data.extend_from_slice(&(self.user_comment_list.len() as u32).to_le_bytes());
        for comment in self.user_comment_list {
            let comment = comment.as_bytes();
            data.extend_from_slice(&(comment.len() as u32).to_le_bytes());
            data.extend_from_slice(comment);
        }

        data
    }
}
