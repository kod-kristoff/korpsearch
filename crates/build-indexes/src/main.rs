use std::{collections::HashSet, fs, path::PathBuf};

use clap::Parser;
use env_logger::Env;
use korpsearch::{corpus::Corpus, corpus_reader::CorpusReaderOptions, index::Index};
use log::LevelFilter;
use miette::IntoDiagnostic;
use options::Options;

mod options;

fn main() -> miette::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();
    let FILE_SUFFIXES: HashSet<&'static str> = HashSet::from_iter(["bz2", "csv"]);
    let args = Options::parse();
    dbg!(&args);
    let mut corpus_id = PathBuf::from(&args.corpus);

    while let Some(ext) = corpus_id.extension() {
        if FILE_SUFFIXES.contains(&ext.to_str().unwrap_or("")) {
            corpus_id.set_extension("");
        }
    }

    let corpus_dir = args
        .base_dir
        .join(&corpus_id)
        .with_extension(Corpus::DIR_SUFFIX);
    println!("corpus_dir = {}", corpus_dir.display());
    let index_dir = args
        .base_dir
        .join(&corpus_id)
        .with_extension(Index::DIR_SUFFIX);
    println!("index_dir = {}", index_dir.display());
    let mut corpus_file = args.base_dir.join(args.corpus);
    if !corpus_file.is_file() {
        let mut candidates = Vec::new();
        for suffix in FILE_SUFFIXES.iter() {
            let candidate_name = format!(
                "{}.{}",
                corpus_file.file_name().unwrap().to_str().unwrap(),
                suffix
            );
            for entry in std::fs::read_dir(corpus_file.parent().unwrap()).into_diagnostic()? {
                let entry = entry.into_diagnostic()?;
                let path = entry.path();
                if path.is_file() {
                    if let Some(filename) = path.file_name() {
                        if filename == candidate_name.as_str() {
                            println!("found candidate: {}", path.display());
                            candidates.push(path);
                        }
                    }
                }
            }
        }
        if candidates.is_empty() {
            eprintln!("Found no source files",);
            std::process::exit(1);
        }
        if candidates.len() > 1 {
            let candidates_str: Vec<String> =
                candidates.iter().map(|p| p.display().to_string()).collect();
            eprintln!(
                "Too many possible source files: {}",
                candidates_str.join(", ")
            );
            std::process::exit(2);
        }
        corpus_file = candidates.remove(0);
    }

    if args.corpus_index {
        fs::create_dir_all(&corpus_dir).into_diagnostic()?;
        let options = CorpusReaderOptions {
            no_reversed_features: args.no_reversed_features,
            no_sentence_feature: args.no_sentence_feature,
        };
        Corpus::build(&corpus_dir, &corpus_file, options)?;
    }
    Ok(())
}
