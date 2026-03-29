/// Generates Docker-style random names (adjective-noun).
pub fn generate_name() -> String {
    use std::hash::{Hash, Hasher};

    let adjectives = [
        "brave", "calm", "clever", "eager", "fair", "gentle", "happy",
        "keen", "lively", "merry", "noble", "proud", "quick", "sharp",
        "swift", "tender", "vivid", "warm", "wise", "bold", "bright",
        "cool", "daring", "elegant", "fierce", "grand", "humble",
        "jolly", "kind", "loyal", "modest", "neat", "patient", "quiet",
        "ready", "smooth", "steady", "tough", "witty",
    ];

    let nouns = [
        "falcon", "heron", "osprey", "eagle", "crane", "robin", "finch",
        "sparrow", "wren", "lark", "dove", "raven", "swift", "owl",
        "hawk", "puffin", "pelican", "condor", "magpie", "jay",
        "tern", "ibis", "kite", "merlin", "oriole", "plover",
        "quail", "shrike", "stork", "thrush",
    ];

    // Use thread-local random state seeded from time + thread ID
    let mut hasher = std::hash::DefaultHasher::new();
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
        .hash(&mut hasher);
    std::thread::current().id().hash(&mut hasher);
    let hash = hasher.finish();

    let adj = adjectives[(hash as usize) % adjectives.len()];
    let noun = nouns[((hash >> 32) as usize) % nouns.len()];

    format!("{adj}-{noun}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_adjective_noun_format() {
        let name = generate_name();
        assert!(name.contains('-'), "expected hyphen in '{name}'");
        let parts: Vec<&str> = name.split('-').collect();
        assert_eq!(parts.len(), 2);
        assert!(!parts[0].is_empty());
        assert!(!parts[1].is_empty());
    }
}
