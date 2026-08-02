//! Bundled species catalog.
//!
//! Users no longer start with someone else's plants -- they add their own.
//! But asking a person to know their plant's "moisture class" is a bad
//! first experience, so adding a plant searches this catalog and pre-fills
//! the care classes plus the uses/significance/fun-fact copy.
//!
//! The data is embedded in the binary at compile time, so lookup works
//! entirely offline and there is no file to ship alongside the exe.

use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

use crate::models::{default_space_id, FertilizeGroup, Light, MoistureClass, PlantProfile};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogEntry {
    pub id: String,
    pub common_name: String,
    /// Other names people actually search by ("money plant", "kadi patta").
    #[serde(default)]
    pub aliases: Vec<String>,
    pub scientific_name: String,
    pub category: String,
    pub light: Light,
    pub moisture_class: MoistureClass,
    pub fertilize_group: FertilizeGroup,
    #[serde(default)]
    pub typically_hanging: bool,
    #[serde(default)]
    pub uses: String,
    #[serde(default)]
    pub significance: String,
    #[serde(default)]
    pub fun_fact: String,
}

impl CatalogEntry {
    /// Builds a plant profile straight from the species defaults. Used by
    /// tests to exercise the schedule engine against every species, and
    /// handy for anything that needs a plant without user edits applied.
    pub fn to_profile(&self) -> PlantProfile {
        PlantProfile {
            id: self.id.clone(),
            common_name: self.common_name.clone(),
            scientific_name: self.scientific_name.clone(),
            category: self.category.clone(),
            light: self.light,
            moisture_class: self.moisture_class,
            fertilize_group: self.fertilize_group,
            is_hanging: self.typically_hanging,
            notes: String::new(),
            inferred: false,
            space_id: default_space_id(),
            uses: self.uses.clone(),
            significance: self.significance.clone(),
            fun_fact: self.fun_fact.clone(),
            // Species defaults carry no per-pot adjustment; that is something
            // the owner discovers about their own plant and sets later.
            water_interval_adjust: 0,
        }
    }
}

static CATALOG: OnceLock<Vec<CatalogEntry>> = OnceLock::new();

pub fn all() -> &'static [CatalogEntry] {
    CATALOG.get_or_init(|| {
        let raw = include_str!("../data/catalog.json");
        serde_json::from_str(raw).expect("bundled catalog.json is malformed")
    })
}

pub fn get(id: &str) -> Option<&'static CatalogEntry> {
    all().iter().find(|e| e.id == id)
}

/// Ranked substring search over name, aliases and scientific name.
///
/// Ranking matters more than cleverness here: someone typing "mint" should
/// see Mint before Indian Borage (whose alias list mentions mint), so an
/// exact or prefix hit on the common name outranks a mid-string alias hit.
pub fn search(query: &str, limit: usize) -> Vec<&'static CatalogEntry> {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        let mut all_sorted: Vec<_> = all().iter().collect();
        all_sorted.sort_by(|a, b| a.common_name.cmp(&b.common_name));
        all_sorted.truncate(limit);
        return all_sorted;
    }

    let mut scored: Vec<(u8, &'static CatalogEntry)> = all()
        .iter()
        .filter_map(|e| score(e, &q).map(|s| (s, e)))
        .collect();

    // Best score first, then alphabetical so results are stable.
    scored.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.common_name.cmp(&b.1.common_name)));
    scored.into_iter().take(limit).map(|(_, e)| e).collect()
}

fn score(entry: &CatalogEntry, q: &str) -> Option<u8> {
    let name = entry.common_name.to_lowercase();
    if name == q {
        return Some(0);
    }
    if name.starts_with(q) {
        return Some(1);
    }
    if entry.aliases.iter().any(|a| a.to_lowercase() == q) {
        return Some(2);
    }
    if entry.aliases.iter().any(|a| a.to_lowercase().starts_with(q)) {
        return Some(3);
    }
    let sci = entry.scientific_name.to_lowercase();
    if sci.starts_with(q) {
        return Some(4);
    }
    if name.contains(q) {
        return Some(5);
    }
    if entry.aliases.iter().any(|a| a.to_lowercase().contains(q)) {
        return Some(6);
    }
    if sci.contains(q) || entry.category.to_lowercase().contains(q) {
        return Some(7);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_parses_and_has_unique_ids() {
        let entries = all();
        assert!(entries.len() >= 140, "catalog unexpectedly small");
        let mut ids: Vec<_> = entries.iter().map(|e| e.id.as_str()).collect();
        ids.sort_unstable();
        let before = ids.len();
        ids.dedup();
        assert_eq!(before, ids.len(), "duplicate catalog ids");
    }

    /// The catalog is hand-authored, and the realistic mistake at this size is
    /// adding a species that is already present under a slightly different
    /// name. Unique ids alone would not catch that, but it would show up to
    /// the user as two near-identical search results.
    #[test]
    fn catalog_entries_are_distinct_species() {
        for (label, mut values) in [
            ("common name", all().iter().map(|e| e.common_name.to_lowercase()).collect::<Vec<_>>()),
            ("scientific name", all().iter().map(|e| e.scientific_name.to_lowercase()).collect::<Vec<_>>()),
        ] {
            values.sort();
            let before = values.len();
            values.dedup();
            assert_eq!(before, values.len(), "duplicate {label} in catalog");
        }
    }

    #[test]
    fn knowledge_copy_has_no_em_dashes() {
        for e in all() {
            for (field, text) in [("uses", &e.uses), ("significance", &e.significance), ("fun_fact", &e.fun_fact)] {
                assert!(!text.contains('\u{2014}'), "{} has an em dash in {field}", e.id);
            }
        }
    }

    #[test]
    fn common_name_outranks_incidental_alias_match() {
        // "mint" appears in Indian Borage's aliases (mexican mint), but the
        // plant actually called Mint must come first.
        let hits = search("mint", 5);
        assert_eq!(hits[0].common_name, "Mint");
    }

    #[test]
    fn finds_plants_by_local_and_common_aliases() {
        assert_eq!(search("kadi patta", 3)[0].id, "curry-leaf");
        assert_eq!(search("money plant", 3)[0].id, "golden-pothos");
        assert_eq!(search("mother in laws tongue", 3)[0].id, "snake-plant");
    }

    #[test]
    fn every_entry_has_knowledge_copy() {
        for e in all() {
            assert!(!e.uses.is_empty(), "{} missing uses", e.id);
            assert!(!e.significance.is_empty(), "{} missing significance", e.id);
            assert!(!e.fun_fact.is_empty(), "{} missing fun_fact", e.id);
        }
    }
}
