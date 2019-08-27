use std::fs;
use std::io::Cursor;
use byteorder::{BigEndian, ReadBytesExt};

struct Mobi {
    contents: Vec<u8>,
    header: Header,
    records: Record,
    palmdoc: PalmDocHeader,
    mobi: MobiHeader,
    exth: ExtHeader,
}
impl Mobi {
}
enum HeaderData {
    Name,
    Attributes,
    Version,
    Created,
    Modified,
    Backup,
    Modnum,
    AppInfoId,
    SortInfoId,
    Typ_e,
    Creator,
    UniqueIdSeed,
    NextRecordListId,
    NumOfRecords,
}
enum PalmDocHeaderData {
    Compression,
    TextLength,
    RecordCount,
    RecordSize,
    EncryptionType,
}

#[derive(Debug)]
struct Header {
    name: String,
    attributes: i16,
    version: i16,
    created: u32,
    modified: u32,
    backup: u32,
    modnum: u32,
    app_info_id: u32,
    sort_info_id: u32,
    typ_e: String,
    creator: String,
    unique_id_seed: u32,
    next_record_list_id: u32,
    num_of_records: u16,
}
impl Header {
    fn parse(content: &Vec<u8>) -> Header {
        Header {
            name: Header::get_headers_string(content, HeaderData::Name),
            attributes: Header::get_headers_i16(content, HeaderData::Attributes),
            version: Header::get_headers_i16(content, HeaderData::Version),
            created: Header::get_headers_u32(content, HeaderData::Created),
            modified: Header::get_headers_u32(content, HeaderData::Modified),
            backup: Header::get_headers_u32(content, HeaderData::Backup),
            modnum: Header::get_headers_u32(content, HeaderData::Modnum),
            app_info_id: Header::get_headers_u32(content, HeaderData::AppInfoId),
            sort_info_id: Header::get_headers_u32(content, HeaderData::SortInfoId),
            typ_e: Header::get_headers_string(content, HeaderData::Typ_e),
            creator: Header::get_headers_string(content, HeaderData::Creator),
            unique_id_seed: Header::get_headers_u32(content, HeaderData::UniqueIdSeed),
            next_record_list_id: Header::get_headers_u32(content, HeaderData::NextRecordListId),
            num_of_records: Header::get_headers_u16(content, HeaderData::NumOfRecords),
        }
    }
    fn get_headers_i16(content: &Vec<u8>, header: HeaderData) -> i16 {
        let mut reader = Cursor::new(content);
        match header {
            HeaderData::Attributes => reader.set_position(32),
            HeaderData::Version => reader.set_position(34),
            _ => {}
        }
        reader.read_i16::<BigEndian>().unwrap()
    }
    fn get_headers_u16(content: &Vec<u8>, header: HeaderData) -> u16 {
        let mut reader = Cursor::new(content);
        match header {
            HeaderData::NumOfRecords => reader.set_position(76),
            _ => {}
        }
        reader.read_u16::<BigEndian>().unwrap()
    }    
    fn get_headers_u32(content: &Vec<u8>, header: HeaderData) -> u32 {
        let mut reader = Cursor::new(content);
        match header {
            HeaderData::Created => reader.set_position(36),
            HeaderData::Modified => reader.set_position(40),
            HeaderData::Backup => reader.set_position(44),
            HeaderData::Modnum => reader.set_position(48),
            HeaderData::AppInfoId => reader.set_position(52),
            HeaderData::SortInfoId => reader.set_position(56),
            HeaderData::UniqueIdSeed => reader.set_position(68),
            HeaderData::NextRecordListId => reader.set_position(72),
            _ => {}
        }
        reader.read_u32::<BigEndian>().unwrap()
    }
    fn get_headers_string(content: &Vec<u8>, header: HeaderData) -> String {
        match header {
            HeaderData::Name => u8_as_string(&content[0..32]),
            HeaderData::Typ_e => u8_as_string(&content[60..64]),
            HeaderData::Creator => u8_as_string(&content[64..68]),
            _ => String::new(),
        }
    }
}
#[derive(Debug)]
struct PalmDocHeader {
    compression: u16,
    text_length: u32,
    record_count: u16,
    record_size: u16,
    encryption_type: u16,
}
impl PalmDocHeader {
    fn parse(content: &Vec<u8>, num_of_records: u16) -> PalmDocHeader {
        PalmDocHeader {
            compression: PalmDocHeader::get_headers_u16(content, PalmDocHeaderData::Compression, num_of_records),
            text_length: PalmDocHeader::get_headers_u32(content, PalmDocHeaderData::TextLength, num_of_records),
            record_count: PalmDocHeader::get_headers_u16(content, PalmDocHeaderData::RecordCount, num_of_records),
            record_size: PalmDocHeader::get_headers_u16(content, PalmDocHeaderData::RecordSize, num_of_records),
            encryption_type: PalmDocHeader::get_headers_u16(content, PalmDocHeaderData::EncryptionType, num_of_records),
        }
    }
    fn get_headers_u16(content: &Vec<u8>, pdheader: PalmDocHeaderData, num_of_records: u16) -> u16 {
        let mut reader = Cursor::new(content);
        match pdheader {
            PalmDocHeaderData::Compression => reader.set_position(80 + (num_of_records*8) as u64),
            PalmDocHeaderData::RecordCount => reader.set_position(88 + (num_of_records*8) as u64),
            PalmDocHeaderData::RecordSize => reader.set_position(90 + (num_of_records*8) as u64),
            PalmDocHeaderData::EncryptionType => reader.set_position(92 + (num_of_records*8) as u64),
            _ => {}
        }
        reader.read_u16::<BigEndian>().unwrap()
    }    
    fn get_headers_u32(content: &Vec<u8>, pdheader: PalmDocHeaderData, num_of_records: u16) -> u32 {
        let mut reader = Cursor::new(content);
        match pdheader {
            PalmDocHeaderData::TextLength => reader.set_position(84 + (num_of_records*8) as u64),
            _ => {}
        }
        reader.read_u32::<BigEndian>().unwrap()
    }    
}
fn u8_as_string(byte_vec: &[u8]) -> String {
    let mut out_str = String::new();
    for byte in byte_vec {
        out_str.push(*byte as char);
    }
    out_str
}

struct MobiHeader;
struct ExtHeader;

#[derive(Debug)]
struct Record {
    record_data_offset: u32,
    id: u32,
}
impl Record {
    fn new() -> Record {
        Record {
            record_data_offset: 0,
            id: 0,
        }
    }
    fn parse_record(reader: &mut Cursor<&Vec<u8>>) -> Record {
        let mut record = Record::new();
        let record_data_offset = reader.read_u32::<BigEndian>().unwrap();
        let id = reader.read_u32::<BigEndian>().unwrap();
        Record {
            record_data_offset,
            id,
        }
    }
    fn parse_records(content: &Vec<u8>, num_of_records: u16) -> Vec<Record> {
        let mut reader = Cursor::new(content);
        reader.set_position(78);

        let mut records = vec![];
        for i in 0..num_of_records {
            let record = Record::parse_record(&mut reader);
            records.push(record);
        }
        records
    }
    fn read(&self, content: &Vec<u8>, record_num: usize, records: &Vec<Record>) -> String {
        let next_record = &records[record_num + 1];
        u8_as_string(&content[self.record_data_offset as usize..next_record.record_data_offset as usize])
    }
    // TODO
    // lz77 decompression
}

fn main() {
    let content = fs::read("/home/wojtek/Downloads/ania.mobi").unwrap();
    let x = Header::parse(&content);
    let p = PalmDocHeader::parse(&content, x.num_of_records);
    let r = Record::parse_records(&content, x.num_of_records);
    println!("{:#?}", x);
    println!("{:#?}", p);
    let mut record_num = 0;
    for record in &r {
        println!("{}", record.read(&content, record_num, &r));
        record_num += 1;
    }
}

// https://docs.python.org/2/library/struct.html
// https://github.com/crabhit/mobi-python/blob/master/mobi/__init__.py