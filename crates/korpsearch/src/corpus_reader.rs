use core::panicking::panic;
use std::{
    fs::File,
    io::{self, BufRead},
    path::{Path, PathBuf},
};

use crate::shared::feature::{EMPTY, FValue, Feature, FeatureFromStrError};

// pub type Header = &[Feature];
pub type Token = Vec<FValue>;
pub type Sentence = Vec<Token>;

pub trait CorpusReader {
    fn header(&self) -> &[Feature];
    fn sentences<'a>(&'a mut self) -> Box<dyn Iterator<Item = Sentence>>;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct CorpusReaderOptions {
    pub no_reversed_features: bool,
    pub no_sentence_feature: bool,
}

pub fn corpus_reader(
    path: &Path,
    description: &'static str,
    args: CorpusReaderOptions,
) -> Result<AugumentedReader, CorpusReaderError> {
    let suffix = uncompressed_suffix(path);
    log::debug!("suffix = {}", suffix);
    let reader = match suffix.as_str() {
        "csv" => CsvReader::from_path(path, description)?,
        _ => return Err(CorpusReaderError::UnsupportedFileType(suffix)),
    };
    Ok(AugumentedReader::new(Box::new(reader), args))
}

#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub enum CorpusReaderError {
    #[error("Cannot find a corpus reader for file type: {0}")]
    UnsupportedFileType(String),
    #[error("Cannot open file '{path}'.")]
    CannotOpenFile { path: PathBuf, source: io::Error },
    #[error("Ill-formed feature")]
    IllformedFeature(#[from] FeatureFromStrError),
}

impl CorpusReaderError {
    pub fn cannot_open_file<P: Into<PathBuf>>(path: P, source: io::Error) -> Self {
        Self::CannotOpenFile {
            path: path.into(),
            source,
        }
    }
}
pub struct AugumentedReader {
    wrapped: Box<dyn CorpusReader>,
    args: CorpusReaderOptions,
}

impl AugumentedReader {
    pub fn new(reader: Box<dyn CorpusReader>, args: CorpusReaderOptions) -> Self {
        Self {
            wrapped: reader,
            args,
        }
    }
}
impl<CR: CorpusReader> AugumentedReader<CR> {
    fn header(&self) -> &[Feature] {
        self.wrapped.header()
    }

    fn sentences<'a>(&'a mut self) -> AugumentedSenteces<'a> {
        AugumentedSenteces {
            args: &self.args,
            sentences: Box::new(self.wrapped.sentences()) as Box<dyn Iterator<Item = Sentence>>,
        }
    }
}

struct AugumentedSenteces<'a> {
    args: &'a CorpusReaderOptions,
    sentences: Box<dyn Iterator<Item = Sentence>>,
}

impl<'a> Iterator for AugumentedSenteces<'a> {
    type Item = Sentence;

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}

pub struct CsvReader<R> {
    reader: io::BufReader<R>,
    description: &'static str,
    header: Vec<Feature>,
}

impl CsvReader<CsvReader<File>> {
    pub fn from_path(
        path: &Path,
        description: &'static str,
    ) -> Result<CsvReader<File>, CorpusReaderError> {
        Ok(CsvReader::new(
            File::open(path).map_err(|source| CorpusReaderError::cannot_open_file(path, source))?,
            description,
        )?)
    }
}
impl<R: io::Read> CsvReader<R> {
    pub fn new(reader: R, description: &'static str) -> Result<Self, CorpusReaderError> {
        let mut reader = io::BufReader::new(reader);
        let mut header_line = String::new();
        reader.read_line(&mut header_line).unwrap();
        let mut header: Vec<Feature> = Vec::new();
        for f in header_line.trim().split('\t') {
            header.push(f.try_into()?);
        }
        Ok(Self {
            reader,
            description,
            header,
        })
    }
    // pub fn new_boxed(path: &Path, description: &'static str) -> Box<dyn CorpusReader> {
    //     Box::new(Self::new(path,description))
    // }
}

impl<R: io::Read> CorpusReader for CsvReader<R> {
    fn header(&self) -> &[Feature] {
        &self.header
    }
    fn sentences<'a>(&'a mut self) -> Box<dyn Iterator<Item = Sentence>> {
        Box::new(CsvReaderSentences {
            sentence: None,
            reader: &mut self.reader,
            n_feats: self.header.len(),
            line_nr: 0,
        })
    }
}

struct CsvReaderSentences<'a, R> {
    sentence: Option<Sentence>,
    reader: &'a mut io::BufReader<R>,
    n_feats: usize,
    line_nr: usize,
}

impl<'a, R: io::Read> Iterator for CsvReaderSentences<'a, R> {
    type Item = Sentence;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let mut line = String::new();
            let read = self.reader.read_line(&mut line).unwrap();
            if read == 0 {
                break;
            }
            self.line_nr += 1;
            if line.starts_with("# ") {
                if let Some(sentence) = self.sentence.take() {
                    return Some(sentence);
                }
            } else if !line.is_empty() {
                let mut token: Token = line.trim().split("\t").map(FValue::from).collect();
                if token.len() < self.n_feats {
                    for _ in 0..(self.n_feats - token.len()) {
                        token.push(EMPTY.clone());
                    }
                }
                if token.len() > self.n_feats {
                    log::error!(
                        "Line {}, too many columns (>{}): {:?}",
                        self.line_nr,
                        self.n_feats,
                        token
                    );
                    panic!("Line contains too many columns.")
                }
                if let Some(sentence) = &mut self.sentence {
                    sentence.push(token);
                } else {
                    self.sentence = Some(vec![token]);
                }
            }
        }
        if let Some(sentence) = self.sentence.take() {
            Some(sentence)
        } else {
            None
        }
    }
}

pub fn uncompressed_suffix(path: &Path) -> String {
    if let Some(ext) = path.extension() {
        if CompressedFileReader::SUPPORTED_TYPES.contains(&ext.to_str().unwrap()) {
            match path.with_extension("").extension() {
                None => "".into(),
                Some(ext) => ext
                    .to_str()
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| String::new()),
            }
        } else {
            ext.to_str()
                .map(ToString::to_string)
                .unwrap_or_else(|| String::new())
        }
    } else {
        "".into()
    }
}

pub struct CompressedFileReader {}

impl CompressedFileReader {
    pub const SUPPORTED_TYPES: &[&str] = &["gz", "bz2", "xz"];

    pub fn new(path: &Path) -> Self {
        Self {}
    }
}
