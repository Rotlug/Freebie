//! Helper methods to slugify game strings

use unicode_normalization::UnicodeNormalization;

pub trait SlugExt {
    /// Get the slug (uri-friendly) version of a string.
    /// This is used to look up games in igdb.
    fn slug(&self) -> String;

    /// Get an ultra-short slug version of a string.
    /// This is useful if the regular `slug` did not have any matches on igdb.
    fn ultra_slug(&self) -> String;

    /// Replaces all chars in `from` with the string in `to`.
    fn multi_replace(&self, from: &str, to: &str) -> String;

    /// Removes game editions like "Deluxe", "Premium", from the end of the string.
    fn remove_game_editions(&self) -> &str;

    /// Checks if the string contains any of the chars in `chars`.
    fn contains_any(&self, chars: &str) -> bool;

    /// Cuts the string at the first occurance of any of the chars in `chars`.
    fn cut_first(&self, chars: &str) -> &str;
}

impl<T: AsRef<str>> SlugExt for T {
    fn slug(&self) -> String {
        let mut result = self.cut_first("–,").to_string();
        result = result.replace(" -", "");

        // Remove non-ascii characters
        result = result.nfd().filter(char::is_ascii).collect();

        // Remove other special characters and clean up other stuff
        result = result
            .multi_replace("#$%&()*+,./:;<=>?@[\\]^_`{|}~!", "")
            .to_ascii_lowercase()
            .replace("goty", "game of the year");

        // Collapse whitespace
        result = result.split_whitespace().collect::<Vec<&str>>().join(" ");
        result.multi_replace(" '", "-")
    }

    fn ultra_slug(&self) -> String {
        let slug = self.slug();
        let slug = slug.remove_game_editions();

        if slug.contains_any("+-&,") {
            slug.cut_first("+-&,");
        } else {
            slug.cut_first(":-,/");
        }

        slug.trim_matches('-').to_string()
    }

    fn multi_replace(&self, from: &str, to: &str) -> String {
        let mut result = self.as_ref().to_string();
        for string in from.chars() {
            result = result.replace(string, to);
        }
        result
    }

    fn remove_game_editions(&self) -> &str {
        const EDITIONS: [&str; 4] = [
            "digital deluxe",
            "deluxe",
            "premium",
            "the one who waits bundle",
        ];

        for edition in EDITIONS {
            if self.as_ref().contains(edition) {
                return self.as_ref().split(edition).next().unwrap();
            }
        }

        self.as_ref().trim()
    }

    fn contains_any(&self, chars: &str) -> bool {
        for char in chars.chars() {
            if self.as_ref().contains(char) {
                return true;
            }
        }

        false
    }

    fn cut_first(&self, chars: &str) -> &str {
        let mut result = self.as_ref();
        for char in chars.chars() {
            result = result.split(char).next().unwrap();
        }

        result
    }
}

mod tests {
    #[allow(unused)] // this actually is used, idk why i get this warning
    use super::*;

    #[test]
    fn slug_doesnt_change_slug() {
        let slugs = [
            "elden-ring",
            "red-dead-redemption-2",
            "ghost-of-tsushima-directors-cut",
            "stray",
            "hollow-knight-silksong",
            "resident-evil-requiem",
            "god-of-war-ragnarok",
            "forza-horizon-6",
            "cyberpunk-2077",
            "baldurs-gate-3",
            "the-legend-of-zelda-tears-of-the-kingdom",
            "alan-wake-2",
            "starfield",
            "final-fantasy-7-rebirth",
            "grand-theft-auto-v",
            "hades-2",
            "it-takes-two",
            "the-last-of-us-part-1",
            "cuphead",
            "disco-elysium",
            "outer-wilds",
            "persona-5-royal",
        ];

        for slug in slugs {
            assert_eq!(slug.slug(), slug);
        }
    }
}
