//! Scryfall API integration for on-demand card fetching.
//!
//! Fetches card data from `api.scryfall.com` and converts it to our
//! internal `CardDef` representation. Results are cached to disk in
//! `.scryfall_cache/` so each card is only fetched once.
//!
//! # Usage
//!
//! ```ignore
//! let mut fetcher = ScryfallFetcher::new(".scryfall_cache");
//! let card_def = fetcher.fetch_card("Lightning Bolt")?;
//! ```
//!
//! # Rate limiting
//!
//! Scryfall asks for 50-100ms between requests. The fetcher enforces a
//! 100ms delay between API calls automatically.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use crate::card::*;
use crate::game::CardDatabase;
use crate::mana::{Color, ManaCost};

// ---------------------------------------------------------------------------
// Scryfall API response types
// ---------------------------------------------------------------------------

/// Subset of Scryfall card JSON we care about.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScryfallCard {
    pub name: String,
    #[serde(default)]
    pub mana_cost: Option<String>,
    #[serde(default)]
    pub type_line: Option<String>,
    #[serde(default)]
    pub oracle_text: Option<String>,
    #[serde(default)]
    pub power: Option<String>,
    #[serde(default)]
    pub toughness: Option<String>,
    #[serde(default)]
    pub keywords: Vec<String>,
    #[serde(default)]
    pub loyalty: Option<String>,
    #[serde(default)]
    pub colors: Vec<String>,
    #[serde(default)]
    pub color_identity: Vec<String>,
    /// Scryfall layout: "normal", "transform", "modal_dfc", "split", etc.
    #[serde(default)]
    pub layout: Option<String>,
    /// For double-faced / split cards, the individual faces.
    #[serde(default)]
    pub card_faces: Option<Vec<ScryfallCardFace>>,
    /// Whether this card is legal in Commander.
    #[serde(default)]
    pub legalities: Option<ScryfallLegalities>,
}

/// A face of a double-faced or split card.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScryfallCardFace {
    pub name: String,
    #[serde(default)]
    pub mana_cost: Option<String>,
    #[serde(default)]
    pub type_line: Option<String>,
    #[serde(default)]
    pub oracle_text: Option<String>,
    #[serde(default)]
    pub power: Option<String>,
    #[serde(default)]
    pub toughness: Option<String>,
    #[serde(default)]
    pub keywords: Vec<String>,
    #[serde(default)]
    pub loyalty: Option<String>,
    #[serde(default)]
    pub colors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScryfallLegalities {
    #[serde(default)]
    pub commander: Option<String>,
    #[serde(default)]
    pub standard: Option<String>,
    #[serde(default)]
    pub modern: Option<String>,
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub enum ScryfallError {
    /// HTTP or network error.
    Network(String),
    /// Card not found on Scryfall.
    NotFound(String),
    /// JSON parse error.
    Parse(String),
    /// File I/O error.
    Io(std::io::Error),
}

impl std::fmt::Display for ScryfallError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScryfallError::Network(msg) => write!(f, "Scryfall network error: {msg}"),
            ScryfallError::NotFound(name) => write!(f, "card not found on Scryfall: '{name}'"),
            ScryfallError::Parse(msg) => write!(f, "Scryfall JSON parse error: {msg}"),
            ScryfallError::Io(err) => write!(f, "Scryfall cache I/O error: {err}"),
        }
    }
}

impl std::error::Error for ScryfallError {}

impl From<std::io::Error> for ScryfallError {
    fn from(err: std::io::Error) -> Self {
        ScryfallError::Io(err)
    }
}

// ---------------------------------------------------------------------------
// Fetcher with disk cache
// ---------------------------------------------------------------------------

/// Fetches cards from Scryfall with a local disk cache.
///
/// Each card is cached as a JSON file in the cache directory. Subsequent
/// lookups for the same card name return the cached version without
/// hitting the network.
pub struct ScryfallFetcher {
    cache_dir: PathBuf,
    last_request: Option<Instant>,
    next_card_id: CardId,
}

impl ScryfallFetcher {
    /// Create a new fetcher with the given cache directory.
    /// The directory will be created if it doesn't exist.
    pub fn new<P: AsRef<Path>>(cache_dir: P) -> Self {
        let cache_dir = cache_dir.as_ref().to_path_buf();
        ScryfallFetcher {
            cache_dir,
            last_request: None,
            next_card_id: 10_000, // Start high to avoid collisions with sample IDs
        }
    }

    /// Create a fetcher with a custom starting card ID.
    pub fn with_start_id<P: AsRef<Path>>(cache_dir: P, start_id: CardId) -> Self {
        let cache_dir = cache_dir.as_ref().to_path_buf();
        ScryfallFetcher {
            cache_dir,
            last_request: None,
            next_card_id: start_id,
        }
    }

    fn allocate_id(&mut self) -> CardId {
        let id = self.next_card_id;
        self.next_card_id += 1;
        id
    }

    /// Sanitize a card name into a safe filename.
    fn cache_filename(name: &str) -> String {
        name.chars()
            .map(|c| match c {
                '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
                ' ' => '_',
                c => c.to_ascii_lowercase(),
            })
            .collect::<String>()
            + ".json"
    }

    /// Try to load a card from the disk cache.
    fn load_cached(&self, name: &str) -> Option<ScryfallCard> {
        let path = self.cache_dir.join(Self::cache_filename(name));
        let data = fs::read_to_string(&path).ok()?;
        serde_json::from_str(&data).ok()
    }

    /// Save a card to the disk cache.
    fn save_cache(&self, name: &str, card: &ScryfallCard) -> Result<(), ScryfallError> {
        fs::create_dir_all(&self.cache_dir)?;
        let path = self.cache_dir.join(Self::cache_filename(name));
        let json = serde_json::to_string_pretty(card)
            .map_err(|e| ScryfallError::Parse(e.to_string()))?;
        fs::write(path, json)?;
        Ok(())
    }

    /// Enforce rate limiting (100ms between requests).
    fn rate_limit(&mut self) {
        if let Some(last) = self.last_request {
            let elapsed = last.elapsed();
            let min_delay = Duration::from_millis(100);
            if elapsed < min_delay {
                thread::sleep(min_delay - elapsed);
            }
        }
        self.last_request = Some(Instant::now());
    }

    /// Fetch a card's raw Scryfall data by exact name.
    /// Returns cached data if available, otherwise queries the API.
    pub fn fetch_raw(&mut self, name: &str) -> Result<ScryfallCard, ScryfallError> {
        // Check cache first
        if let Some(cached) = self.load_cached(name) {
            return Ok(cached);
        }

        // Rate limit
        self.rate_limit();

        // Query Scryfall API
        let url = format!(
            "https://api.scryfall.com/cards/named?exact={}",
            urlencode(name)
        );

        let response = ureq::get(&url)
            .set("User-Agent", "mtg-gto/0.1 (Rust game engine)")
            .call()
            .map_err(|e| match &e {
                ureq::Error::Status(404, _) => ScryfallError::NotFound(name.to_string()),
                _ => ScryfallError::Network(e.to_string()),
            })?;

        let card: ScryfallCard = response
            .into_json()
            .map_err(|e| ScryfallError::Parse(e.to_string()))?;

        // Cache for next time
        self.save_cache(name, &card)?;

        Ok(card)
    }

    /// Fetch a card and convert it to our internal `CardDef`.
    pub fn fetch_card_def(&mut self, name: &str) -> Result<CardDef, ScryfallError> {
        let raw = self.fetch_raw(name)?;
        let id = self.allocate_id();
        Ok(scryfall_to_card_def(&raw, id))
    }

    /// Import an entire deck list from a string, fetching all cards from
    /// Scryfall. Returns (CardDatabase, deck as Vec<CardId>, Option<commander_id>).
    ///
    /// Supports the format:
    /// ```text
    /// ~~Commanders~~
    /// 1 Card Name
    ///
    /// ~~Mainboard~~
    /// 1 Card Name
    /// ...
    /// ```
    pub fn import_deck(
        &mut self,
        deck_text: &str,
    ) -> Result<DeckImportResult, ScryfallError> {
        let mut db = CardDatabase::new();
        let mut deck: Vec<CardId> = Vec::new();
        let mut commander_id: Option<CardId> = None;
        // Map from card name -> CardId to avoid duplicate fetches
        let mut name_to_id: HashMap<String, CardId> = HashMap::new();

        let mut in_commander_section = false;
        let mut fetch_errors: Vec<String> = Vec::new();

        for line in deck_text.lines() {
            let trimmed = line.trim();

            // Skip empty lines
            if trimmed.is_empty() {
                continue;
            }

            // Section headers
            if trimmed.starts_with("~~") && trimmed.ends_with("~~") {
                let section = trimmed.trim_matches('~').trim().to_lowercase();
                in_commander_section = section.contains("commander");
                continue;
            }

            // Skip comment lines
            if trimmed.starts_with("//") || trimmed.starts_with('#') {
                continue;
            }

            // Parse "N Card Name"
            let first_space = match trimmed.find(|c: char| c.is_whitespace()) {
                Some(i) => i,
                None => continue,
            };
            let (qty_str, rest) = trimmed.split_at(first_space);
            let quantity: u32 = match qty_str.parse() {
                Ok(n) => n,
                Err(_) => continue,
            };
            let card_name = rest.trim();
            if card_name.is_empty() {
                continue;
            }

            // Handle double-faced / split card names: "Front // Back" -> use "Front"
            let lookup_name = if card_name.contains(" // ") {
                card_name.split(" // ").next().unwrap_or(card_name).trim()
            } else {
                card_name
            };

            // Fetch or reuse card
            let card_id = if let Some(&existing_id) = name_to_id.get(lookup_name) {
                existing_id
            } else {
                match self.fetch_card_def(lookup_name) {
                    Ok(mut def) => {
                        let id = def.id;
                        // Use the full deck list name for display
                        if card_name.contains(" // ") {
                            def.name = card_name.to_string();
                        }
                        db.insert(def);
                        name_to_id.insert(lookup_name.to_string(), id);
                        id
                    }
                    Err(e) => {
                        fetch_errors.push(format!("{}: {}", card_name, e));
                        continue;
                    }
                }
            };

            // Add to deck
            for _ in 0..quantity {
                deck.push(card_id);
            }

            // Track commander
            if in_commander_section {
                commander_id = Some(card_id);
            }
        }

        Ok(DeckImportResult {
            db,
            deck,
            commander_id,
            errors: fetch_errors,
        })
    }
}

/// Result of importing a deck via Scryfall.
pub struct DeckImportResult {
    /// Card definitions for all successfully fetched cards.
    pub db: CardDatabase,
    /// The full deck list as CardIds (including commander).
    pub deck: Vec<CardId>,
    /// The commander's CardId, if a ~~Commanders~~ section was present.
    pub commander_id: Option<CardId>,
    /// Cards that failed to fetch (name + error message).
    pub errors: Vec<String>,
}

// ---------------------------------------------------------------------------
// Card conversion: ScryfallCard -> CardDef
// ---------------------------------------------------------------------------

/// Convert a Scryfall card to our internal `CardDef`.
pub fn scryfall_to_card_def(card: &ScryfallCard, id: CardId) -> CardDef {
    // For double-faced cards, use the front face data
    let (mana_cost_str, type_line, oracle_text, power_str, toughness_str, keywords, loyalty_str) =
        if let Some(faces) = &card.card_faces {
            if let Some(front) = faces.first() {
                (
                    front.mana_cost.as_deref().or(card.mana_cost.as_deref()),
                    front.type_line.as_deref().unwrap_or(""),
                    front.oracle_text.as_deref().unwrap_or(""),
                    front.power.as_deref(),
                    front.toughness.as_deref(),
                    &front.keywords,
                    front.loyalty.as_deref(),
                )
            } else {
                (
                    card.mana_cost.as_deref(),
                    card.type_line.as_deref().unwrap_or(""),
                    card.oracle_text.as_deref().unwrap_or(""),
                    card.power.as_deref(),
                    card.toughness.as_deref(),
                    &card.keywords,
                    card.loyalty.as_deref(),
                )
            }
        } else {
            (
                card.mana_cost.as_deref(),
                card.type_line.as_deref().unwrap_or(""),
                card.oracle_text.as_deref().unwrap_or(""),
                card.power.as_deref(),
                card.toughness.as_deref(),
                &card.keywords,
                card.loyalty.as_deref(),
            )
        };

    // Parse mana cost
    let mana_cost = mana_cost_str.and_then(|s| {
        if s.is_empty() {
            None
        } else {
            ManaCost::parse(s)
        }
    });

    // Parse type line
    let (supertypes, card_types, subtypes) = parse_type_line(type_line);

    // Parse keywords
    let kw_abilities = parse_keywords(keywords);

    // Parse power/toughness
    let power = power_str.and_then(|s| s.parse::<i32>().ok());
    let toughness = toughness_str.and_then(|s| s.parse::<i32>().ok());

    // Parse loyalty
    let starting_loyalty = loyalty_str.and_then(|s| s.parse::<u32>().ok());

    // Parse mana abilities for lands
    let mana_abilities = parse_land_mana_abilities(type_line, oracle_text);

    // Parse spell effect from oracle text
    let spell_effect = if card_types.contains(&CardType::Instant)
        || card_types.contains(&CardType::Sorcery)
    {
        parse_spell_effect(oracle_text)
    } else {
        None
    };

    // Parse ETB triggered abilities from oracle text
    let triggered_abilities = parse_triggered_abilities(oracle_text);

    CardDef {
        id,
        name: card.name.clone(),
        mana_cost,
        card_types,
        supertypes,
        subtypes,
        keywords: kw_abilities,
        power,
        toughness,
        mana_abilities,
        spell_effect,
        activated_abilities: Vec::new(),
        triggered_abilities,
        static_abilities: Vec::new(),
        starting_loyalty,
        enters_tapped: oracle_text.contains("enters the battlefield tapped")
            || oracle_text.contains("enters tapped"),
        oracle_text: oracle_text.to_string(),
        dynamic_power: None,
        dynamic_toughness: None,
        cost_reduction: None,
    }
}

// ---------------------------------------------------------------------------
// Type line parser
// ---------------------------------------------------------------------------

/// Parse a Scryfall type line like "Legendary Creature — Human Soldier"
/// into (supertypes, card_types, subtypes).
fn parse_type_line(type_line: &str) -> (Vec<Supertype>, Vec<CardType>, Vec<Subtype>) {
    let mut supertypes = Vec::new();
    let mut card_types = Vec::new();
    let mut subtypes = Vec::new();

    // Split on " — " (em dash) or " - " (hyphen) using split_once
    // for robustness (avoids manual byte-offset arithmetic for the
    // multi-byte em dash character).
    let (main_part, sub_part) = if let Some((main, sub)) = type_line.split_once(" — ") {
        (main, Some(sub))
    } else if let Some((main, sub)) = type_line.split_once(" - ") {
        (main, Some(sub))
    } else {
        (type_line, None)
    };

    // Parse main types
    for word in main_part.split_whitespace() {
        match word {
            "Legendary" => supertypes.push(Supertype::Legendary),
            "Basic" => supertypes.push(Supertype::Basic),
            "Snow" => supertypes.push(Supertype::Snow),
            "Creature" => card_types.push(CardType::Creature),
            "Instant" => card_types.push(CardType::Instant),
            "Sorcery" => card_types.push(CardType::Sorcery),
            "Enchantment" => card_types.push(CardType::Enchantment),
            "Artifact" => card_types.push(CardType::Artifact),
            "Planeswalker" => card_types.push(CardType::Planeswalker),
            "Land" => card_types.push(CardType::Land),
            _ => {} // "Tribal", "World", etc. — ignored
        }
    }

    // Parse subtypes
    if let Some(subs) = sub_part {
        for sub in subs.split_whitespace() {
            let sub = sub.trim();
            if !sub.is_empty() {
                // Check if it's a basic land subtype (implies mana production)
                subtypes.push(Subtype(sub.to_string()));
            }
        }
    }

    (supertypes, card_types, subtypes)
}

// ---------------------------------------------------------------------------
// Keyword parser
// ---------------------------------------------------------------------------

/// Map Scryfall keyword strings to our KeywordAbility enum.
fn parse_keywords(keywords: &[String]) -> Vec<KeywordAbility> {
    let mut result = Vec::new();
    for kw in keywords {
        if let Some(ability) = match kw.as_str() {
            "Flying" => Some(KeywordAbility::Flying),
            "First strike" => Some(KeywordAbility::FirstStrike),
            "Double strike" => Some(KeywordAbility::DoubleStrike),
            "Deathtouch" => Some(KeywordAbility::Deathtouch),
            "Haste" => Some(KeywordAbility::Haste),
            "Hexproof" => Some(KeywordAbility::Hexproof),
            "Indestructible" => Some(KeywordAbility::Indestructible),
            "Lifelink" => Some(KeywordAbility::Lifelink),
            "Menace" => Some(KeywordAbility::Menace),
            "Reach" => Some(KeywordAbility::Reach),
            "Trample" => Some(KeywordAbility::Trample),
            "Vigilance" => Some(KeywordAbility::Vigilance),
            "Defender" => Some(KeywordAbility::Defender),
            "Flash" => Some(KeywordAbility::Flash),
            "Fear" => Some(KeywordAbility::Fear),
            "Intimidate" => Some(KeywordAbility::Intimidate),
            "Shroud" => Some(KeywordAbility::Shroud),
            "Protection" => Some(KeywordAbility::Protection),
            _ => None, // Unrecognized keywords silently ignored
        } {
            result.push(ability);
        }
    }
    result
}

// ---------------------------------------------------------------------------
// Land mana ability parser
// ---------------------------------------------------------------------------

/// Determine what mana a land can produce from its type line and oracle text.
fn parse_land_mana_abilities(type_line: &str, oracle_text: &str) -> Vec<ManaAbility> {
    let mut abilities = Vec::new();

    // Basic land subtypes imply mana production
    let tl = type_line.to_lowercase();
    if tl.contains("plains") {
        abilities.push(ManaAbility::TapForColor(Color::White));
    }
    if tl.contains("mountain") {
        abilities.push(ManaAbility::TapForColor(Color::Red));
    }
    if tl.contains("forest") {
        abilities.push(ManaAbility::TapForColor(Color::Green));
    }
    if tl.contains("island") {
        abilities.push(ManaAbility::TapForColor(Color::Blue));
    }
    if tl.contains("swamp") {
        abilities.push(ManaAbility::TapForColor(Color::Black));
    }

    // If we already found abilities from subtypes, return
    if !abilities.is_empty() {
        return abilities;
    }

    // Parse oracle text for common mana ability patterns
    let text = oracle_text.to_lowercase();

    if text.contains("{t}: add one mana of any color")
        || text.contains("{t}: add {c} or one mana of any color")
        || text.contains("add one mana of any color")
    {
        abilities.push(ManaAbility::TapForAny);
    } else if text.contains("{t}: add {c}") || text.contains("{t}: add one colorless") {
        abilities.push(ManaAbility::TapForColorless);
    }

    // Check for specific color production in oracle text
    let color_patterns = [
        ("{w}", Color::White),
        ("{u}", Color::Blue),
        ("{b}", Color::Black),
        ("{r}", Color::Red),
        ("{g}", Color::Green),
    ];

    if text.contains("{t}: add ") {
        for (symbol, color) in &color_patterns {
            if text.contains(&format!("add {symbol}"))
                || text.contains(&format!("add {symbol} or"))
            {
                if !abilities.iter().any(|a| matches!(a, ManaAbility::TapForAny)) {
                    abilities.push(ManaAbility::TapForColor(*color));
                }
            }
        }

        // Deduplicate into TapForChoice if multiple colors
        if abilities.len() > 1
            && abilities
                .iter()
                .all(|a| matches!(a, ManaAbility::TapForColor(_)))
        {
            let colors: Vec<Color> = abilities
                .iter()
                .filter_map(|a| {
                    if let ManaAbility::TapForColor(c) = a {
                        Some(*c)
                    } else {
                        None
                    }
                })
                .collect();
            abilities = vec![ManaAbility::TapForChoice(colors)];
        }
    }

    abilities
}

// ---------------------------------------------------------------------------
// Spell effect parser
// ---------------------------------------------------------------------------

/// Attempt to parse oracle text into a spell Effect.
/// Handles common patterns; falls back to Effect::Unimplemented for complex cards.
fn parse_spell_effect(oracle_text: &str) -> Option<Effect> {
    let text = oracle_text.to_lowercase();

    // Counter target spell
    if text.contains("counter target spell") {
        return Some(Effect::Counter {
            target: TargetSpec::AnySpell,
        });
    }

    // Destroy all creatures
    if text.contains("destroy all creatures") {
        return Some(Effect::DestroyAll);
    }

    // Destroy target creature
    if text.contains("destroy target creature") {
        return Some(Effect::DestroyTarget {
            target: TargetSpec::AnyCreature,
        });
    }

    // Destroy target nonland permanent
    if text.contains("destroy target nonland permanent") {
        return Some(Effect::DestroyTarget {
            target: TargetSpec::AnyNonlandPermanent,
        });
    }

    // Exile target creature
    if text.contains("exile target creature") {
        return Some(Effect::ExileTarget {
            target: TargetSpec::AnyCreature,
        });
    }

    // Deal N damage to any target / target creature or player
    if let Some(amount) = parse_damage_amount(&text, "deals") {
        let target = if text.contains("any target")
            || text.contains("target creature or player")
            || text.contains("creature or planeswalker")
        {
            TargetSpec::CreatureOrPlayer
        } else if text.contains("target creature") {
            TargetSpec::AnyCreature
        } else if text.contains("target player") || text.contains("target opponent") {
            TargetSpec::AnyPlayer
        } else if text.contains("each creature") {
            TargetSpec::EachCreature
        } else {
            TargetSpec::CreatureOrPlayer
        };
        return Some(Effect::DealDamage { amount, target });
    }

    // "deal N damage" pattern (without "deals")
    if let Some(amount) = parse_damage_amount(&text, "deal") {
        let target = if text.contains("any target")
            || text.contains("target creature or player")
        {
            TargetSpec::CreatureOrPlayer
        } else if text.contains("target creature") {
            TargetSpec::AnyCreature
        } else if text.contains("target player") || text.contains("target opponent") {
            TargetSpec::AnyPlayer
        } else {
            TargetSpec::CreatureOrPlayer
        };
        return Some(Effect::DealDamage { amount, target });
    }

    // Draw N cards
    if let Some(count) = extract_number_before(&text, "card") {
        if text.contains("draw") {
            return Some(Effect::DrawCards { count });
        }
    }

    // Gain N life
    if let Some(amount) = extract_number_before(&text, "life") {
        if text.contains("gain") {
            return Some(Effect::GainLife { amount });
        }
    }

    // Return target to hand (bounce)
    if text.contains("return target")
        && (text.contains("to its owner's hand") || text.contains("to their owner's hand"))
    {
        let target = if text.contains("creature") {
            TargetSpec::AnyCreature
        } else if text.contains("nonland permanent") {
            TargetSpec::AnyNonlandPermanent
        } else {
            TargetSpec::AnyPermanent
        };
        return Some(Effect::BounceTo {
            zone: ZoneType::Hand,
            target,
        });
    }

    // Target creature gets +N/+N or -N/-N
    if let Some((p, t)) = parse_buff(&text) {
        if p > 0 || t > 0 {
            return Some(Effect::Buff {
                power: p,
                toughness: t,
                until_eot: text.contains("until end of turn"),
            });
        } else {
            return Some(Effect::Debuff {
                power: p,
                toughness: t,
                until_eot: text.contains("until end of turn"),
            });
        }
    }

    // Add mana
    if text.starts_with("add ") || text.contains("{t}: add ") {
        if let Some(amount) = count_mana_symbols(&text) {
            return Some(Effect::AddMana {
                color: None,
                amount,
            });
        }
    }

    // Discard
    if text.contains("discard") && text.contains("card") {
        let count = extract_number_before(&text, "card").unwrap_or(1);
        let target = if text.contains("target player") || text.contains("target opponent") {
            TargetSpec::Opponent
        } else {
            TargetSpec::Controller
        };
        return Some(Effect::DiscardCards { count, target });
    }

    // If we got here, it's too complex to parse automatically
    if !oracle_text.is_empty() {
        Some(Effect::Unimplemented(oracle_text.to_string()))
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// Triggered ability parser
// ---------------------------------------------------------------------------

/// Parse simple triggered abilities from oracle text.
fn parse_triggered_abilities(oracle_text: &str) -> Vec<TriggeredAbility> {
    let text = oracle_text.to_lowercase();
    let mut abilities = Vec::new();

    // "When ~ enters the battlefield, ..."
    if text.contains("enters the battlefield") || text.contains("enters, ") {
        if let Some(effect) = parse_etb_effect(&text) {
            abilities.push(TriggeredAbility {
                trigger: TriggerCondition::EntersBattlefield,
                effect,
                description: oracle_text.to_string(),
            });
        }
    }

    abilities
}

/// Parse the effect portion of an ETB trigger.
fn parse_etb_effect(text: &str) -> Option<Effect> {
    // Find the part after "enters the battlefield" or "enters,"
    let after_etb = if let Some(idx) = text.find("enters the battlefield") {
        &text[idx + "enters the battlefield".len()..]
    } else if let Some(idx) = text.find("enters, ") {
        &text[idx + "enters, ".len()..]
    } else {
        return None;
    };

    let after = after_etb.trim_start_matches(|c: char| c == ',' || c == ' ');

    if after.contains("draw") {
        let count = extract_number_before(after, "card").unwrap_or(1);
        return Some(Effect::DrawCards { count });
    }

    if after.contains("gain") && after.contains("life") {
        let amount = extract_number_before(after, "life").unwrap_or(1);
        return Some(Effect::GainLife { amount });
    }

    if after.contains("destroy target creature") {
        return Some(Effect::DestroyTarget {
            target: TargetSpec::AnyCreature,
        });
    }

    if let Some(amount) = parse_damage_amount(after, "deal") {
        return Some(Effect::DealDamage {
            amount,
            target: TargetSpec::AnyPlayer,
        });
    }

    // Complex ETB — record as unimplemented
    Some(Effect::Unimplemented(after.to_string()))
}

// ---------------------------------------------------------------------------
// Text parsing helpers
// ---------------------------------------------------------------------------

/// Extract a damage amount from text like "deals 3 damage" or "deal 2 damage".
fn parse_damage_amount(text: &str, verb: &str) -> Option<u32> {
    let pattern = format!("{verb} ");
    if let Some(idx) = text.find(&pattern) {
        let after = &text[idx + pattern.len()..];
        let num_str: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
        if let Ok(n) = num_str.parse() {
            return Some(n);
        }
    }

    // Try "N damage" without verb prefix
    for word in text.split_whitespace() {
        if let Ok(n) = word.parse::<u32>() {
            // Check if "damage" follows nearby
            if let Some(idx) = text.find(word) {
                let after = &text[idx + word.len()..];
                if after.trim_start().starts_with("damage") {
                    return Some(n);
                }
            }
        }
    }

    None
}

/// Extract a number that appears before a word, e.g., "draw 3 cards" -> 3.
fn extract_number_before(text: &str, after_word: &str) -> Option<u32> {
    // Look for patterns like "draw two cards", "draw a card", "draw 3 cards"
    let words = ["a ", "an ", "one ", "two ", "three ", "four ", "five ", "six ", "seven "];
    let nums = [1, 1, 1, 2, 3, 4, 5, 6, 7];

    for (word, &num) in words.iter().zip(nums.iter()) {
        let pattern = format!("{word}{after_word}");
        if text.contains(&pattern) {
            return Some(num);
        }
    }

    // Try numeric pattern "N word"
    for i in 1..=20 {
        let pattern = format!("{i} {after_word}");
        if text.contains(&pattern) {
            return Some(i);
        }
    }

    None
}

/// Parse buff/debuff from text: +N/+N, -N/-N, +N/-N, or -N/+N.
fn parse_buff(text: &str) -> Option<(i32, i32)> {
    for &sign_p in &["+", "-"] {
        for &sign_t in &["+", "-"] {
            for p in 0..=10 {
                for t in 0..=10 {
                    let pattern = format!("{sign_p}{p}/{sign_t}{t}");
                    if text.contains(&pattern) {
                        let p_val = if sign_p == "-" { -(p as i32) } else { p as i32 };
                        let t_val = if sign_t == "-" { -(t as i32) } else { t as i32 };
                        return Some((p_val, t_val));
                    }
                }
            }
        }
    }
    None
}

/// Count mana symbols in text like "add {B}{B}{B}" -> 3.
/// Only counts actual mana symbols ({W}, {U}, {B}, {R}, {G}, {C}),
/// not other braced tokens like {T} or ability costs.
fn count_mana_symbols(text: &str) -> Option<u32> {
    let mana_symbols = ["{w}", "{u}", "{b}", "{r}", "{g}", "{c}"];
    let count: u32 = mana_symbols.iter()
        .map(|s| text.matches(s).count() as u32)
        .sum();
    if count > 0 { Some(count.min(10)) } else { None }
}

/// URL-encode a string for use in query parameters.
/// Handles common characters in MTG card names (spaces, apostrophes, commas,
/// colons, etc.) using percent-encoding.
fn urlencode(s: &str) -> String {
    let mut result = String::with_capacity(s.len() * 2);
    for c in s.chars() {
        match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => result.push(c),
            ' ' => result.push('+'),
            _ => {
                for byte in c.to_string().as_bytes() {
                    result.push_str(&format!("%{:02X}", byte));
                }
            }
        }
    }
    result
}
