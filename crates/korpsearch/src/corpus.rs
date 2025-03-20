use std::{
    collections::HashSet,
    fs::File,
    io,
    path::{Path, PathBuf},
};

use crate::{
    corpus_reader::{CorpusReader, CorpusReaderError, CorpusReaderOptions, corpus_reader},
    shared::feature::FValue,
};

pub struct Corpus {}

impl Corpus {
    pub const DIR_SUFFIX: &'static str = "corpus-rs";
    pub const FEATURES_FILE: &'static str = "features.cfg";
    pub const FEATURES_PREFIX: &'static str = "features:";
    pub const SENTENCES_PATH: &'static str = "sentences";

    pub fn build(
        basedir: &Path,
        corpusfile: &Path,
        args: CorpusReaderOptions,
    ) -> Result<(), CorpusBuildError> {
        log::debug!("Building corpus index from file: {}", corpusfile.display());

        let corpus = corpus_reader(corpusfile, "Collecting strings", args)?;
        let feature_path = basedir.join(Corpus::FEATURES_FILE);
        let out =
            io::BufWriter::new(File::create(&feature_path).map_err(|source| {
                CorpusBuildError::failed_to_create_file(&feature_path, source)
            })?);
        serde_json::to_writer(out, corpus.header())
            .map_err(|source| CorpusBuildError::failed_to_write_json(&feature_path, source))?;

        let stringsets: Vec<HashSet<FValue>> =
            corpus.header().iter().map(|_| HashSet::new()).collect();
        let mut n_sentences = 0;
        let mut n_tokens = 0;
        for sentence in corpus.sentences(){
            n_sentences +=1;
            for token in sentence {
                n_tokens += 1;
                for strings, value in stringsets.iter().zip(token) {
                    strings.insert(value);
                }
            }}
        log::debug!(" --> read {} distinct strings, {} sentences, {} tokens",stringsets.iter().map(HashSet::len).sum(),n_sentences,n_tokens)
        Ok(())
    }
}

#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub enum CorpusBuildError {
    #[error("Unknown error: {0}")]
    Unknown(String),
    #[error(transparent)]
    FailedReadingCorpus(#[from] CorpusReaderError),
    #[error("Failed to create file '{path}'")]
    FailedToCreateFile { path: PathBuf, source: io::Error },
    #[error("Failed to write JSON to '{path}'")]
    FailedToWriteJson {
        path: PathBuf,
        source: serde_json::Error,
    },
}

impl CorpusBuildError {
    pub fn failed_to_create_file<P: Into<PathBuf>>(path: P, source: io::Error) -> Self {
        Self::FailedToCreateFile {
            path: path.into(),
            source,
        }
    }
    pub fn failed_to_write_json<P: Into<PathBuf>>(path: P, source: serde_json::Error) -> Self {
        Self::FailedToWriteJson {
            path: path.into(),
            source,
        }
    }
}
