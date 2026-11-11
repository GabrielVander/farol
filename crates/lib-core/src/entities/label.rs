const MAX_LABEL_LENGTH: usize = 63;

/// Why a DNS label (one dot-separated part of a name) is invalid.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LabelError {
    Empty,
    TooLong,
    InvalidCharacter(char),
    HyphenAtEdge,
}

pub(crate) fn validate_labels(name: &str) -> Result<(), LabelError> {
    name.split('.').try_for_each(validate_label)
}

fn validate_label(label: &str) -> Result<(), LabelError> {
    validate_length(label)?;
    validate_characters(label)?;
    validate_edges(label)
}

fn validate_length(label: &str) -> Result<(), LabelError> {
    if label.is_empty() {
        return Err(LabelError::Empty);
    }

    if label.len() > MAX_LABEL_LENGTH {
        return Err(LabelError::TooLong);
    }

    Ok(())
}

fn validate_characters(label: &str) -> Result<(), LabelError> {
    match label.chars().find(|c| !is_label_character(*c)) {
        Some(invalid) => Err(LabelError::InvalidCharacter(invalid)),
        None => Ok(()),
    }
}

fn validate_edges(label: &str) -> Result<(), LabelError> {
    if label.starts_with('-') || label.ends_with('-') {
        return Err(LabelError::HyphenAtEdge);
    }

    Ok(())
}

fn is_label_character(character: char) -> bool {
    character.is_ascii_alphanumeric() || character == '-'
}
