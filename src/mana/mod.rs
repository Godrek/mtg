use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::{Add, AddAssign};

/// The five colors of Magic plus colorless/generic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Color {
    White,
    Blue,
    Black,
    Red,
    Green,
}

impl Color {
    pub const ALL: [Color; 5] = [
        Color::White,
        Color::Blue,
        Color::Black,
        Color::Red,
        Color::Green,
    ];
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Color::White => write!(f, "W"),
            Color::Blue => write!(f, "U"),
            Color::Black => write!(f, "B"),
            Color::Red => write!(f, "R"),
            Color::Green => write!(f, "G"),
        }
    }
}

/// A mana cost, representing what must be paid to cast a spell or activate an ability.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ManaCost {
    pub generic: u32,
    pub white: u32,
    pub blue: u32,
    pub black: u32,
    pub red: u32,
    pub green: u32,
}

impl ManaCost {
    pub fn zero() -> Self {
        ManaCost {
            generic: 0,
            white: 0,
            blue: 0,
            black: 0,
            red: 0,
            green: 0,
        }
    }

    pub fn new(generic: u32, white: u32, blue: u32, black: u32, red: u32, green: u32) -> Self {
        ManaCost {
            generic,
            white,
            blue,
            black,
            red,
            green,
        }
    }

    /// Converted mana cost / mana value.
    pub fn cmc(&self) -> u32 {
        self.generic + self.white + self.blue + self.black + self.red + self.green
    }

    /// Get the amount of a specific color required.
    pub fn color_amount(&self, color: Color) -> u32 {
        match color {
            Color::White => self.white,
            Color::Blue => self.blue,
            Color::Black => self.black,
            Color::Red => self.red,
            Color::Green => self.green,
        }
    }

    /// Parse a mana cost string like "{2}{W}{U}" or "{3}{B}{B}".
    pub fn parse(s: &str) -> Option<ManaCost> {
        let mut cost = ManaCost::zero();
        let mut chars = s.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '{' {
                let mut symbol = String::new();
                for inner in chars.by_ref() {
                    if inner == '}' {
                        break;
                    }
                    symbol.push(inner);
                }
                match symbol.as_str() {
                    "W" => cost.white += 1,
                    "U" => cost.blue += 1,
                    "B" => cost.black += 1,
                    "R" => cost.red += 1,
                    "G" => cost.green += 1,
                    "X" => {} // X costs handled separately
                    n => {
                        if let Ok(v) = n.parse::<u32>() {
                            cost.generic += v;
                        }
                    }
                }
            }
        }
        Some(cost)
    }

    /// Colors present in this mana cost.
    pub fn colors(&self) -> Vec<Color> {
        let mut colors = Vec::new();
        if self.white > 0 {
            colors.push(Color::White);
        }
        if self.blue > 0 {
            colors.push(Color::Blue);
        }
        if self.black > 0 {
            colors.push(Color::Black);
        }
        if self.red > 0 {
            colors.push(Color::Red);
        }
        if self.green > 0 {
            colors.push(Color::Green);
        }
        colors
    }
}

impl fmt::Display for ManaCost {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.generic > 0 || self.cmc() == 0 {
            write!(f, "{{{}}}", self.generic)?;
        }
        for _ in 0..self.white {
            write!(f, "{{W}}")?;
        }
        for _ in 0..self.blue {
            write!(f, "{{U}}")?;
        }
        for _ in 0..self.black {
            write!(f, "{{B}}")?;
        }
        for _ in 0..self.red {
            write!(f, "{{R}}")?;
        }
        for _ in 0..self.green {
            write!(f, "{{G}}")?;
        }
        Ok(())
    }
}

/// A mana pool — the mana a player currently has available.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManaPool {
    pub white: u32,
    pub blue: u32,
    pub black: u32,
    pub red: u32,
    pub green: u32,
    pub colorless: u32,
}

impl ManaPool {
    pub fn empty() -> Self {
        ManaPool {
            white: 0,
            blue: 0,
            black: 0,
            red: 0,
            green: 0,
            colorless: 0,
        }
    }

    pub fn total(&self) -> u32 {
        self.white + self.blue + self.black + self.red + self.green + self.colorless
    }

    pub fn get(&self, color: Color) -> u32 {
        match color {
            Color::White => self.white,
            Color::Blue => self.blue,
            Color::Black => self.black,
            Color::Red => self.red,
            Color::Green => self.green,
        }
    }

    pub fn get_mut(&mut self, color: Color) -> &mut u32 {
        match color {
            Color::White => &mut self.white,
            Color::Blue => &mut self.blue,
            Color::Black => &mut self.black,
            Color::Red => &mut self.red,
            Color::Green => &mut self.green,
        }
    }

    pub fn add_color(&mut self, color: Color, amount: u32) {
        *self.get_mut(color) += amount;
    }

    /// Check if this pool can pay a given mana cost.
    /// Returns true if the cost is payable (colored requirements met + enough total for generic).
    pub fn can_pay(&self, cost: &ManaCost) -> bool {
        // Check each colored requirement
        for &color in &Color::ALL {
            if self.get(color) < cost.color_amount(color) {
                return false;
            }
        }
        // Check that remaining mana covers the generic cost
        let remaining: u32 = Color::ALL
            .iter()
            .map(|&c| self.get(c) - cost.color_amount(c))
            .sum::<u32>()
            + self.colorless;
        remaining >= cost.generic
    }

    /// Pay a mana cost from this pool. Returns false if unable to pay.
    /// Uses a simple greedy strategy: pay colored costs first, then generic from colorless,
    /// then generic from excess colored mana.
    pub fn pay(&mut self, cost: &ManaCost) -> bool {
        if !self.can_pay(cost) {
            return false;
        }
        // Pay colored costs
        for &color in &Color::ALL {
            *self.get_mut(color) -= cost.color_amount(color);
        }
        // Pay generic cost: first from colorless, then from colored
        let mut remaining_generic = cost.generic;
        if remaining_generic > 0 {
            let from_colorless = remaining_generic.min(self.colorless);
            self.colorless -= from_colorless;
            remaining_generic -= from_colorless;
        }
        if remaining_generic > 0 {
            // Pay from colored mana (prefer colors with excess)
            for &color in &Color::ALL {
                if remaining_generic == 0 {
                    break;
                }
                let available = self.get(color);
                let from_color = remaining_generic.min(available);
                *self.get_mut(color) -= from_color;
                remaining_generic -= from_color;
            }
        }
        true
    }

    /// Empty the mana pool (happens at end of each step/phase).
    pub fn drain(&mut self) {
        *self = ManaPool::empty();
    }
}

impl Add for ManaPool {
    type Output = ManaPool;
    fn add(self, rhs: ManaPool) -> ManaPool {
        ManaPool {
            white: self.white + rhs.white,
            blue: self.blue + rhs.blue,
            black: self.black + rhs.black,
            red: self.red + rhs.red,
            green: self.green + rhs.green,
            colorless: self.colorless + rhs.colorless,
        }
    }
}

impl AddAssign for ManaPool {
    fn add_assign(&mut self, rhs: ManaPool) {
        self.white += rhs.white;
        self.blue += rhs.blue;
        self.black += rhs.black;
        self.red += rhs.red;
        self.green += rhs.green;
        self.colorless += rhs.colorless;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mana_cost_parse() {
        let cost = ManaCost::parse("{2}{W}{U}").unwrap();
        assert_eq!(cost.generic, 2);
        assert_eq!(cost.white, 1);
        assert_eq!(cost.blue, 1);
        assert_eq!(cost.cmc(), 4);
    }

    #[test]
    fn test_mana_pool_can_pay() {
        let mut pool = ManaPool::empty();
        pool.white = 2;
        pool.blue = 1;
        pool.red = 1;

        let cost = ManaCost::new(1, 1, 1, 0, 0, 0); // {1}{W}{U}
        assert!(pool.can_pay(&cost));

        let cost2 = ManaCost::new(0, 3, 0, 0, 0, 0); // {W}{W}{W}
        assert!(!pool.can_pay(&cost2));
    }

    #[test]
    fn test_mana_pool_pay() {
        let mut pool = ManaPool::empty();
        pool.white = 2;
        pool.blue = 1;
        pool.red = 1;

        let cost = ManaCost::new(1, 1, 1, 0, 0, 0); // {1}{W}{U}
        assert!(pool.pay(&cost));
        assert_eq!(pool.white, 0); // 1 for colored, 1 for generic
        assert_eq!(pool.blue, 0);
        assert_eq!(pool.red, 1);
    }
}
