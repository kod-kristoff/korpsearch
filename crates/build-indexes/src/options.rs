use std::path::PathBuf;

#[derive(Debug, clap::Parser)]
#[clap(author,version,about,long_about=None)]
pub struct Options {
    /// name of corpus (i.e. without any file suffixes)
    #[clap(short, long, value_name = "CORPUS")]
    pub corpus: String,
    /// build the corpus index
    #[clap(short = 'i', long)]
    pub corpus_index: bool,
    /// directory where to find the corpus
    #[clap(short = 'd', long, value_name = "DIR", default_value = "./corpora/")]
    pub base_dir: PathBuf,
    /// dont't build the 's' feature for sentence breaks (default: do build it)
    #[arg(long)]
    pub no_sentence_feature: bool,
    /// don't build reversed features for suffix search (default: do build them)
    #[arg(long)]
    pub no_reversed_features: bool,
}
