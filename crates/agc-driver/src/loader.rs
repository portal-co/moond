//! Markdown loader for AGC ISA specs.
//!
//! Reads all `.md` files in a directory (typically `ref/block2/`).
//! For each file, looks for a `## Semantics` section followed by a
//! ` ```agc-sem` ... ` ``` ` code block.
//! Parses the DSL with `agc_isa::sem_text::parse_sem()`.
//! Falls back to the builtin spec if no DSL is found.

use std::fs;
use std::path::Path;

use agc_isa::spec::InstrSpecSet;
use agc_isa::sem_text::parse_sem;
use agc_isa::builtin_spec_set;

/// Errors that can occur while loading specs from disk.
#[derive(Debug)]
pub enum LoadError {
    Io(std::io::Error),
    Parse {
        file: String,
        instr: String,
        error: &'static str,
    },
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoadError::Io(e) => write!(f, "I/O error: {e}"),
            LoadError::Parse { file, instr, error } => {
                write!(f, "parse error in {file} ({instr}): {error}")
            }
        }
    }
}

impl From<std::io::Error> for LoadError {
    fn from(e: std::io::Error) -> Self { LoadError::Io(e) }
}

/// Load specs from all `.md` files in `block2_dir`.
///
/// Starts with the built-in spec set and overlays any semantics found in
/// `## Semantics` sections with `agc-sem` code blocks.
pub fn load_from_dir(block2_dir: &Path) -> Result<InstrSpecSet, LoadError> {
    let mut specs = builtin_spec_set();

    let entries = fs::read_dir(block2_dir)?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }

        let content = fs::read_to_string(&path)?;
        let file_name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        if let Some((mnemonic, dsl)) = extract_semantics_block(&content) {
            // Find the matching spec and update its semantics
            if let Some(spec) = specs.by_mnemonic(&mnemonic) {
                let instr_type = spec.instr_type;
                let ops = parse_sem(&dsl).map_err(|e| LoadError::Parse {
                    file: file_name.clone(),
                    instr: mnemonic.clone(),
                    error: e,
                })?;
                if let Some(spec_mut) = specs.by_type_mut(instr_type) {
                    spec_mut.semantics = Some(ops);
                    spec_mut.source_file = Some(file_name);
                }
            }
        }
    }

    Ok(specs)
}

/// Extract the mnemonic and DSL text from a markdown file's `## Semantics` section.
///
/// Looks for a `## Semantics` section containing a code fence with `agc-sem` language tag.
/// The mnemonic is inferred from the first `# MNEMONIC ...` heading in the file.
fn extract_semantics_block(content: &str) -> Option<(String, String)> {
    // Extract the mnemonic from the first heading: `# TC K — ...` → "TC"
    let mnemonic = extract_mnemonic(content)?;

    // Find the `## Semantics` section
    let sem_start = content.find("## Semantics")?;
    let after_sem = &content[sem_start..];

    // Find the ```agc-sem code block
    let block_start = after_sem.find("```agc-sem")?;
    let after_block = &after_sem[block_start + "```agc-sem".len()..];

    // Find the closing ```
    let block_end = after_block.find("```")?;
    let dsl = after_block[..block_end].trim().to_string();

    if dsl.is_empty() {
        return None;
    }

    Some((mnemonic, dsl))
}

/// Extract the first-word mnemonic from the first `# ` heading.
fn extract_mnemonic(content: &str) -> Option<String> {
    for line in content.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("# ") {
            // First word of the heading is the mnemonic (up to space or special char)
            let mnemonic = rest
                .split_whitespace()
                .next()?
                .to_uppercase();
            if !mnemonic.is_empty() {
                return Some(mnemonic);
            }
        }
    }
    None
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_mnemonic_from_heading() {
        let md = "# TC K — Transfer Control to K (Block-2)\n\nSome text.";
        assert_eq!(extract_mnemonic(md), Some("TC".to_string()));
    }

    #[test]
    fn extract_semantics_block_basic() {
        let md = "# CA K — Clear and Add K\n\n## Semantics\n\n```agc-sem\nset A mem\n```\n";
        let result = extract_semantics_block(md);
        assert!(result.is_some());
        let (m, dsl) = result.unwrap();
        assert_eq!(m, "CA");
        assert_eq!(dsl, "set A mem");
    }

    #[test]
    fn extract_no_semantics_returns_none() {
        let md = "# TC K — Transfer Control\n\nNo semantics here.";
        assert!(extract_semantics_block(md).is_none());
    }

    #[test]
    fn builtin_specset_has_all_instrs() {
        let specs = builtin_spec_set();
        assert!(specs.by_mnemonic("TC").is_some());
        assert!(specs.by_mnemonic("CCS").is_some());
        assert!(specs.by_mnemonic("CA").is_some());
    }
}
