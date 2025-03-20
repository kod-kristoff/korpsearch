use once_cell::sync::Lazy;
use regex::Regex;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Feature(String);
#[derive(Debug, Clone)]
pub struct FValue(String);

pub static WORD: Lazy<Feature> = Lazy::new(|| Feature("word".into()));
pub static SENTENCE: Lazy<Feature> = Lazy::new(|| Feature("s".into()));

pub static EMPTY: Lazy<FValue> = Lazy::new(|| FValue("".into()));
pub static START: Lazy<FValue> = Lazy::new(|| FValue("s".into()));

impl TryFrom<&str> for Feature {
    type Error = FeatureFromStrError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        static VALID_FEATURE: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"^[a-z_][a-z_0-9]*$").unwrap());
        if VALID_FEATURE.is_match(value) {
            Ok(Self(value.to_string()))
        } else {
            Err(FeatureFromStrError::IllformedFeature(value.to_string()))
        }
    }
}

#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub enum FeatureFromStrError {
    #[error("Ill-formed feature: {0}")]
    IllformedFeature(String),
}

impl From<&str> for FValue {
    fn from(value: &str) -> Self {
        Self(value.into())
    }
}
