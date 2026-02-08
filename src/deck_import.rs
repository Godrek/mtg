use crate::card::{DeckEntry, Decklist};
use crate::game::CardDatabase;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::fs;
use std::path::Path;

#[derive(Debug)]
pub enum DeckImportError {
    Io(std::io::Error),
    MissingDeckName,
    InvalidLine {
        line_number: usize,
        line: String,
        message: String,
    },
    UnknownCard {
        line_number: usize,
        name: String,
    },
}

impl fmt::Display for DeckImportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeckImportError::Io(err) => write!(f, "failed to read deck file: {err}"),
            DeckImportError::MissingDeckName => write!(f, "deck filename is missing"),
            DeckImportError::InvalidLine {
                line_number,
                line,
                message,
            } => write!(
                f,
                "invalid line {line_number}: {message} (got: '{line}')"
            ),
            DeckImportError::UnknownCard { line_number, name } => {
                write!(f, "unknown card on line {line_number}: '{name}'")
            }
        }
    }
}

impl Error for DeckImportError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            DeckImportError::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for DeckImportError {
    fn from(err: std::io::Error) -> Self {
        DeckImportError::Io(err)
    }
}

pub fn import_deck_from_file<P: AsRef<Path>>(
    path: P,
    card_db: &CardDatabase,
) -> Result<Decklist, DeckImportError> {
    let path = path.as_ref();
    let deck_name = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .filter(|name| !name.trim().is_empty())
        .ok_or(DeckImportError::MissingDeckName)?
        .to_string();

    let contents = fs::read_to_string(path)?;
    let mut entries: Vec<DeckEntry> = Vec::new();
    let mut indices: HashMap<u64, usize> = HashMap::new();

    for (line_number, raw_line) in contents.lines().enumerate() {
        let line_number = line_number + 1;
        let trimmed = raw_line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let first_space = trimmed
            .find(|c: char| c.is_whitespace())
            .ok_or_else(|| DeckImportError::InvalidLine {
                line_number,
                line: raw_line.to_string(),
                message: "missing card name after quantity".to_string(),
            })?;
        let (qty_str, rest) = trimmed.split_at(first_space);
        let quantity: u32 = qty_str.parse().map_err(|_| DeckImportError::InvalidLine {
            line_number,
            line: raw_line.to_string(),
            message: "quantity is not a number".to_string(),
        })?;
        if quantity == 0 {
            return Err(DeckImportError::InvalidLine {
                line_number,
                line: raw_line.to_string(),
                message: "quantity must be greater than zero".to_string(),
            });
        }

        let name = rest.trim();
        if name.is_empty() {
            return Err(DeckImportError::InvalidLine {
                line_number,
                line: raw_line.to_string(),
                message: "card name is empty".to_string(),
            });
        }

        let card_id = card_db
            .find_by_name(name)
            .ok_or_else(|| DeckImportError::UnknownCard {
                line_number,
                name: name.to_string(),
            })?;

        if let Some(index) = indices.get(&card_id).copied() {
            entries[index].quantity += quantity;
        } else {
            indices.insert(card_id, entries.len());
            entries.push(DeckEntry { card_id, quantity });
        }
    }

    Ok(Decklist {
        name: deck_name,
        cards: entries,
    })
}
