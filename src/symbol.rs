#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Symbol {
    Aircraft,
    Balloon,
    Car,
    Digi,
    Gateway,
    Home,
    Motorcycle,
    Person,
    Phone,
    Police,
    Repeater,
    Sailboat,
    Ship,
    Storm,
    Truck,
    Weather,
}

impl std::fmt::Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl TryFrom<(char, char)> for Symbol {
    type Error = ();

    fn try_from(value: (char, char)) -> Result<Self, Self::Error> {
        let (table, code) = value;

        // Only the primary ('/') and alternate ('\') tables are handled here;
        // overlay tables (any other char) use the alternate table's symbols with a visual overlay.
        match (table, code) {
            ('/', '!') | ('\\', '!') => Ok(Self::Police),
            ('/', '#') | ('\\', '#') => Ok(Self::Digi),
            ('/', '$') => Ok(Self::Phone),
            ('/', '&') | ('\\', '&') => Ok(Self::Gateway),
            ('/', '\'') => Ok(Self::Aircraft),
            ('/', '-') | ('\\', '-') => Ok(Self::Home),
            ('/', '<') => Ok(Self::Motorcycle),
            ('/', '>') => Ok(Self::Car),
            ('/', '@') | ('\\', '@') => Ok(Self::Storm),
            ('/', 'O') | ('\\', 'O') => Ok(Self::Balloon),
            ('/', 'R') | ('\\', 'r') => Ok(Self::Repeater),
            ('/', 'Y') | ('\\', 'Y') => Ok(Self::Sailboat),
            ('/', '[') | ('\\', '[') => Ok(Self::Person),
            ('/', '^') | ('\\', '^') => Ok(Self::Aircraft),
            ('/', '_') => Ok(Self::Weather),
            ('/', 'k') | ('\\', 'k') => Ok(Self::Truck),
            ('/', 's') | ('\\', 's') => Ok(Self::Ship),
            _ => Err(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── symbol_label ───────────────────────────────────────────────────

    #[test]
    fn symbol_label_primary_table() {
        assert_eq!(Symbol::try_from(('/', '>')), Ok(Symbol::Car));
        assert_eq!(Symbol::try_from(('/', '-')), Ok(Symbol::Home));
        assert_eq!(Symbol::try_from(('/', '_')), Ok(Symbol::Weather));
        assert_eq!(Symbol::try_from(('/', '&')), Ok(Symbol::Gateway));
        assert_eq!(Symbol::try_from(('/', '[')), Ok(Symbol::Person));
        assert_eq!(Symbol::try_from(('/', '<')), Ok(Symbol::Motorcycle));
    }

    #[test]
    fn symbol_label_alternate_table() {
        assert_eq!(Symbol::try_from(('\\', '-')), Ok(Symbol::Home));
        assert_eq!(Symbol::try_from(('\\', '#')), Ok(Symbol::Digi));
    }

    #[test]
    fn symbol_label_unknown_returns_none() {
        assert_eq!(Symbol::try_from(('/', 'Z')), Err(()));
        assert_eq!(Symbol::try_from(('X', '>')), Err(()));
    }
}
