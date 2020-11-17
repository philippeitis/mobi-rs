//! An implementation of [MOBI](https://wiki.mobileread.com/wiki/MOBI) format data parsing and manipulation, written in Rust.
//!
//! The code is available on [GitHub](https://github.com/wojciechkepka/mobi-rs)
//!
//! License: [*MIT*](https://github.com/wojciechkepka/mobi-rs/blob/master/license)
//!
//! ## Examples
//! ### Access basic info
//! ```no_run
//! use mobi::Mobi;
//! fn main() -> Result<(), std::io::Error> {
//!     let book = vec![0, 0, 0];
//!     // You can either create a Mobi struct from a slice
//!     let m = Mobi::new(&book)?;
//!     // Or from an instance of io::Read
//!     let stdin = std::io::stdin();
//!     let mut handle = stdin.lock();
//!     let m = Mobi::from_read(&mut handle)?;
//!     // Or from filesystem
//!     let m = Mobi::from_path("/some/path/to/book.mobi")?;
//!
//!     let empty = "".to_string();
//!     // Access metadata
//!     let title = m.title().unwrap_or(&empty);
//!     let author = m.author().unwrap_or(&empty);
//!     let publisher = m.publisher().unwrap_or(&empty);
//!     let desc = m.description().unwrap_or(&empty);
//!     let isbn = m.isbn().unwrap_or(&empty);
//!     let pub_date = m.publish_date().unwrap_or(&empty);
//!     let contributor = m.contributor().unwrap_or(&empty);
//!
//!     // Access Headers
//!     let metadata = &m.metadata;
//!     let header = &metadata.header; // Normal Header
//!     let pdheader = &metadata.palmdoc; // PalmDOC Header
//!     let mheader = &metadata.mobi; // MOBI Header
//!     let exth = &metadata.exth; // Extra Header
//!
//!     // Access content
//!     let content = m.content_as_string();
//!
//!     Ok(())
//! }
//! ```
pub(crate) mod book;
#[cfg(feature = "fmt")]
mod display;
/// Module with headers from book containg more extracted data not
/// available through public API.
pub mod headers;
pub(crate) mod lz77;
pub(crate) mod reader;
pub(crate) mod record;
#[cfg(feature = "time")]
use chrono::prelude::*;
use headers::{exth, ExtHeader, Header, MobiHeader, PalmDocHeader, TextEncoding};
pub(crate) use reader::Reader;
pub use record::Record;
use std::{fs, io, io::Read, ops::Range, path::Path};


#[derive(Debug, Default)]
/// Structure that holds parsed ebook information and contents
pub struct Mobi {
    pub raw_content: Vec<u8>,
    pub metadata: MobiMetadata,
    pub records: Vec<Record>,
}

#[derive(Debug, Default)]
/// Structure that holds parsed ebook information
pub struct MobiMetadata {
    pub header: Header,
    pub palmdoc: PalmDocHeader,
    pub mobi: MobiHeader,
    pub exth: ExtHeader,
}

impl MobiMetadata {
    /// Construct a MobiMetadata object from a slice
    pub fn new<B: AsRef<Vec<u8>>>(bytes: B) -> io::Result<MobiMetadata> {
        MobiMetadata::from_reader(&mut Reader::new(bytes.as_ref()))
    }

    /// Construct a MobiMetadata object from passed file path
    pub fn from_path<P: AsRef<Path>>(file_path: P) -> io::Result<MobiMetadata> {
        MobiMetadata::new(&fs::read(file_path)?)
    }

    /// Construct a MobiMetadata object from an object that implements a Read trait
    pub fn from_read<R: Read>(reader: R) -> io::Result<MobiMetadata> {
        let content: Vec<_> = reader.bytes().flatten().collect();
        MobiMetadata::from_reader(&mut Reader::new(&content))
    }

    fn from_reader(mut reader: &mut Reader) -> io::Result<MobiMetadata> {
        let header = Header::parse(&mut reader)?;
        reader.set_num_of_records(header.num_of_records);
        let palmdoc = PalmDocHeader::parse(&mut reader)?;
        let mobi = MobiHeader::parse(&mut reader)?;
        let exth = {
            if mobi.has_exth_header {
                ExtHeader::parse(&mut reader, mobi.header_length)?
            } else {
                ExtHeader::default()
            }
        };

        Ok(MobiMetadata {
            header,
            palmdoc,
            mobi,
            exth,
        })
    }

    /// Returns author record if such exists
    pub fn author(&self) -> Option<&String> {
        self.exth.get_record(exth::ExthRecord::Author)
    }

    /// Returns publisher record if such exists
    pub fn publisher(&self) -> Option<&String> {
        self.exth.get_record(exth::ExthRecord::Publisher)
    }

    /// Returns description record if such exists
    pub fn description(&self) -> Option<&String> {
        self.exth.get_record(exth::ExthRecord::Description)
    }

    /// Returns isbn record if such exists
    pub fn isbn(&self) -> Option<&String> {
        self.exth.get_record(exth::ExthRecord::Isbn)
    }

    /// Returns publish_date record if such exists
    pub fn publish_date(&self) -> Option<&String> {
        self.exth.get_record(exth::ExthRecord::PublishDate)
    }

    /// Returns contributor record if such exists
    pub fn contributor(&self) -> Option<&String> {
        self.exth.get_record(exth::ExthRecord::Contributor)
    }

    /// Returns title record if such exists
    pub fn title(&self) -> Option<&String> {
        self.exth.get_record(exth::ExthRecord::Title)
    }

    /// Returns text encoding used in ebook
    pub fn text_encoding(&self) -> TextEncoding {
        self.mobi.text_encoding()
    }

    /// Returns type of this ebook
    pub fn mobi_type(&self) -> Option<String> {
        self.mobi.mobi_type()
    }

    /// Returns language of the ebook
    pub fn language(&self) -> Option<String> {
        self.mobi.language()
    }

    #[cfg(feature = "time")]
    /// Returns creation datetime
    /// This field is only available using `time` feature
    pub fn created_datetime(&self) -> NaiveDateTime {
        self.header.created_datetime()
    }

    #[cfg(feature = "time")]
    /// Returns modification datetime
    /// This field is only available using `time` feature
    pub fn mod_datetime(&self) -> NaiveDateTime {
        self.header.mod_datetime()
    }

    /// Returns creation time as u32 timestamp
    pub fn created_time(&self) -> u32 {
        self.header.created_datetime()
    }

    /// Returns last modification time as u32 timestamp
    pub fn mod_time(&self) -> u32 {
        self.header.mod_datetime()
    }

    /// Returns compression method used on this file
    pub fn compression(&self) -> String {
        self.palmdoc.compression()
    }
    /// Returns encryption method used on this file
    pub fn encryption(&self) -> String {
        self.palmdoc.encryption()
    }

    /// Returns last readable index of the book
    fn last_index(&self) -> usize {
        (self.palmdoc.record_count - 1) as usize
    }
}

impl Mobi {
    /// Construct a Mobi object from a slice
    pub fn new<B: AsRef<Vec<u8>>>(bytes: B) -> io::Result<Mobi> {
        Mobi::from_reader(&mut Reader::new(bytes.as_ref()))
    }
    /// Construct a Mobi object from passed file path
    pub fn from_path<P: AsRef<Path>>(file_path: P) -> io::Result<Mobi> {
        Mobi::new(&fs::read(file_path)?)
    }
    /// Construct a Mobi object from an object that implements a Read trait
    pub fn from_read<R: Read>(reader: R) -> io::Result<Mobi> {
        // Temporary solution
        let content: Vec<_> = reader.bytes().flatten().collect();
        Mobi::from_reader(&mut Reader::new(&content))
    }

    fn from_reader(reader: &mut Reader) -> io::Result<Mobi> {
        let metadata = MobiMetadata::from_reader(reader)?;
        let records = Record::parse_records(
            reader.content_ref(),
            metadata.header.num_of_records,
            metadata.mobi.extra_bytes,
            metadata.palmdoc.compression_enum(),
        )?;
        Ok(Mobi {
            raw_content: reader.content(),
            metadata,
            records,
        })
    }

    /// Returns author record if such exists
    pub fn author(&self) -> Option<&String> {
        self.metadata.author()
    }
    /// Returns publisher record if such exists
    pub fn publisher(&self) -> Option<&String> {
        self.metadata.publisher()
    }
    /// Returns description record if such exists
    pub fn description(&self) -> Option<&String> {
        self.metadata.description()
    }
    /// Returns isbn record if such exists
    pub fn isbn(&self) -> Option<&String> {
        self.metadata.isbn()
    }
    /// Returns publish_date record if such exists
    pub fn publish_date(&self) -> Option<&String> {
        self.metadata.publish_date()
    }
    /// Returns contributor record if such exists
    pub fn contributor(&self) -> Option<&String> {
        self.metadata.contributor()
    }
    /// Returns title record if such exists
    pub fn title(&self) -> Option<&String> {
        self.metadata.title()
    }
    /// Returns text encoding used in ebook
    pub fn text_encoding(&self) -> TextEncoding {
        self.metadata.text_encoding()
    }
    /// Returns type of this ebook
    pub fn mobi_type(&self) -> Option<String> {
        self.metadata.mobi_type()
    }
    /// Returns language of the ebook
    pub fn language(&self) -> Option<String> {
        self.metadata.language()
    }
    #[cfg(feature = "time")]
    /// Returns creation datetime
    /// This field is only available using `time` feature
    pub fn created_datetime(&self) -> NaiveDateTime {
        self.metadata.created_datetime()
    }
    #[cfg(feature = "time")]
    /// Returns modification datetime
    /// This field is only available using `time` feature
    pub fn mod_datetime(&self) -> NaiveDateTime {
        self.metadata.mod_datetime()
    }

    /// Returns creation time as u32 timestamp
    pub fn created_time(&self) -> u32 {
        self.metadata.created_time()
    }

    /// Returns last modification time as u32 timestamp
    pub fn mod_time(&self) -> u32 {
        self.metadata.mod_time()
    }

    /// Returns compression method used on this file
    pub fn compression(&self) -> String {
        self.metadata.compression()
    }
    /// Returns encryption method used on this file
    pub fn encryption(&self) -> String {
        self.metadata.encryption()
    }

    /// Returns last readable index of the book
    fn last_index(&self) -> usize {
        self.metadata.last_index()
    }

    fn readable_records_range(&self) -> Range<usize> {
        1..self.last_index()
    }

    /// Returns all readable records content decompressed as a String.
    /// There are only two supported encodings in mobi format (UTF8, WIN1252)
    /// and both are losely converted by this function
    pub fn content_as_string(&self) -> String {
        self.readable_records_range()
            .map(|i| self.records[i as usize].to_string(self.text_encoding()))
            .collect()
    }

    /// Returns all readable records content decompressed as a Vec
    pub fn content(&self) -> Vec<u8> {
        self.readable_records_range()
            .map(|i| self.records[i as usize].record_data.clone())
            .flatten()
            .collect()
    }
}
