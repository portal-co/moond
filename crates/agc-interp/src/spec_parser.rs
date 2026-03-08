//! Parse AGC Block-2 reference markdown files into [`InstrSpec`] / [`InstrSpecSet`].
//!
//! # Usage
//!
//! ```no_run
//! use agc_interp::spec_parser::SpecParser;
//!
//! let specs = SpecParser::load("ref/block2").unwrap();
//! println!("Loaded {} instruction specs", specs.len());
//! ```
//!
//! # What is parsed
//!
//! * **`OPCODE_ENCODING.md`** — the instruction encoding tables (mnemonic, order code,
//!   address mode, description).  This is the primary source of encoding metadata.
//! * **`<mnemonic>_<type>.md`** files — per-instruction docs.  The parser extracts the
//!   first-line summary from the `## Summary` / `Summary` section and attaches it to the
//!   matching spec as `description`.

use std::path::{Path, PathBuf};
use std::fs;
use std::str::FromStr;

use crate::spec::{AddrMode, InstrSpec, InstrSpecSet, InstrType, OpcodeFormat};

// ─── Public API ───────────────────────────────────────────────────────────────

/// Error returned by the spec parser.
#[derive(Debug)]
pub struct ParseError {
    pub message: String,
    pub file: Option<PathBuf>,
}

impl ParseError {
    fn new(msg: impl Into<String>) -> Self {
        ParseError { message: msg.into(), file: None }
    }
    fn with_file(mut self, path: &Path) -> Self {
        self.file = Some(path.to_owned());
        self
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(p) = &self.file {
            write!(f, "{}: {}", p.display(), self.message)
        } else {
            f.write_str(&self.message)
        }
    }
}

impl std::error::Error for ParseError {}

pub struct SpecParser;

impl SpecParser {
    /// High-level entry point: load all specs from a `ref/block2`-style directory.
    ///
    /// 1. Starts with the built-in ISA table (so all instructions are always present).
    /// 2. Reads `OPCODE_ENCODING.md` (if present) and updates encoding metadata.
    /// 3. Reads each `<mnemonic>_<type>.md` file and enriches the description.
    pub fn load(block2_dir: impl AsRef<Path>) -> Result<InstrSpecSet, ParseError> {
        let dir = block2_dir.as_ref();

        // Start from the built-in table so unknown files still produce a complete ISA.
        let mut set = InstrSpecSet::builtin();

        // Step 1: parse OPCODE_ENCODING.md for authoritative encoding metadata.
        let encoding_path = dir.join("OPCODE_ENCODING.md");
        if encoding_path.exists() {
            let text = fs::read_to_string(&encoding_path)
                .map_err(|e| ParseError::new(e.to_string()).with_file(&encoding_path))?;
            parse_encoding_doc(&text, &mut set);
        }

        // Step 2: enrich each instruction's description from its individual .md file.
        let entries = fs::read_dir(dir)
            .map_err(|e| ParseError::new(e.to_string()))?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("md") { continue; }
            let fname = path.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_uppercase();

            // Skip index files; only parse per-instruction docs
            if matches!(fname.as_str(),
                "README" | "INDEX" | "OPCODE_ENCODING" | "DIFFERENCES" |
                "ENCODER_DECODER" | "DECODER_DESIGN" | "DECODER_REFACTOR" |
                "MEMORY_MAP" | "BIT_HELPERS" | "BIT_REVERSAL_ADDRESSES" |
                "BIT_REVERSAL_DECISION" | "BIT_ORDERING_VERIFICATION" |
                "BIT_REVERSAL_IMPLEMENTATION" | "BIT_REVERSAL_REQUIRED" |
                "RESUME" | "GO"
            ) { continue; }

            if let Ok(text) = fs::read_to_string(&path) {
                enrich_from_instr_doc(&text, &path, &mut set);
            }
        }

        Ok(set)
    }

    /// Parse only the encoding tables from OPCODE_ENCODING.md text.
    pub fn parse_encoding_doc(text: &str) -> InstrSpecSet {
        let mut set = InstrSpecSet::builtin();
        parse_encoding_doc(text, &mut set);
        set
    }
}

// ─── Encoding doc parser ──────────────────────────────────────────────────────

/// Parse markdown tables from `OPCODE_ENCODING.md` and update `set` in place.
///
/// Tables are identified by having `Mnemonic` as the first column header.
/// Each data row maps: mnemonic → order code → address mode type → description.
fn parse_encoding_doc(text: &str, set: &mut InstrSpecSet) {
    let mut in_table = false;
    let mut header_seen = false;

    for line in text.lines() {
        let line = line.trim();
        if !line.starts_with('|') { in_table = false; header_seen = false; continue; }

        // Detect table header row containing "Mnemonic"
        if line.to_lowercase().contains("mnemonic") && line.contains('|') {
            in_table = true;
            header_seen = false; // next non-separator row is the separator
            continue;
        }

        if !in_table { continue; }

        // Skip separator rows (---|---|-...)
        if line.chars().all(|c| c == '|' || c == '-' || c == ':' || c == ' ') {
            header_seen = true;
            continue;
        }
        if !header_seen { continue; }

        // Parse data row
        let cols: Vec<&str> = line.split('|')
            .map(|c| c.trim())
            .filter(|c| !c.is_empty())
            .collect();

        if cols.len() < 2 { continue; }

        let mnemonic = cols[0];
        if mnemonic.is_empty() || mnemonic.starts_with('-') { continue; }

        // Mnemonic may contain multiple with commas or slashes; take the first.
        let mnemonic = mnemonic.split(|c: char| c == ',' || c == '/').next().unwrap_or(mnemonic).trim();

        let Ok(instr_type) = InstrType::from_str(mnemonic) else { continue; };

        let order_code = cols.get(1).copied().unwrap_or("");
        let addr_type  = cols.get(2).copied().unwrap_or("");
        let desc       = cols.get(3).copied().unwrap_or("");

        let (opcode_format, opcode, quarter) = parse_order_code(order_code);
        let addr_mode = AddrMode::from_str(addr_type).unwrap_or(AddrMode::None);
        let requires_extend = instr_type.requires_extend();

        // Only update if we got useful info; builtin table is already correct.
        let spec = InstrSpec {
            mnemonic: mnemonic.to_string(),
            instr_type,
            addr_mode,
            opcode_format,
            opcode,
            quarter,
            requires_extend,
            description: desc.to_string(),
            source_file: None,
        };

        set.insert(spec);
    }
}

/// Parse an order-code string like `"00."`, `"01.2"`, `"10.0"`, `"none"`.
///
/// Returns `(format, opcode_value, quarter_code)`.
fn parse_order_code(s: &str) -> (OpcodeFormat, u8, Option<u8>) {
    // Take only the first order code if multiple are listed (e.g. "01.2, 01.4, 01.6")
    let s = s.split(',').next().unwrap_or(s).trim();

    let fallback = (OpcodeFormat::Hardware, 0, None);

    if s.is_empty() || s == "-" || s.eq_ignore_ascii_case("none") {
        return fallback;
    }

    // Special full-word encodings like "00.0006", "00.0004", "00.0003"
    // These are Special format instructions.
    if let Some(dot) = s.find('.') {
        let before = &s[..dot];
        let after  = &s[dot + 1..];

        // Parse octal opcode before the dot
        let Ok(opcode) = u8::from_str_radix(before, 8) else { return fallback; };

        if after.is_empty() {
            // "NN." — opcode only, no quarter
            // Determine format by opcode value:
            //   0-7 single digit → Whole3
            //   >= 010 octal (8+) → may be Quarter5
            if opcode <= 7 {
                return (OpcodeFormat::Whole3, opcode, None);
            } else {
                // e.g. "13." or "17." — Quarter5 without explicit quarter
                return (OpcodeFormat::Quarter5, opcode, None);
            }
        }

        // "NN.Q" or "NN.QQQQ" (special)
        if after.len() > 1 {
            // Special instruction (e.g. "00.0006" for EXTEND)
            return (OpcodeFormat::Special, opcode, None);
        }

        let Ok(quarter) = u8::from_str_radix(after, 8) else { return fallback; };

        // Channel instructions: opcode 010 (octal) = 8 decimal
        if opcode == 0o10 {
            return (OpcodeFormat::Channel6, opcode, Some(quarter));
        }

        return (OpcodeFormat::Quarter5, opcode, Some(quarter));
    }

    fallback
}

// ─── Individual instruction doc enricher ─────────────────────────────────────

/// Extract the first bullet or line of the Summary section from an instruction `.md` file
/// and store it as the `description` on the matching spec.
fn enrich_from_instr_doc(text: &str, path: &Path, set: &mut InstrSpecSet) {
    // Infer the instruction type from the H1 heading (first `# ` line).
    let mnemonic = extract_mnemonic_from_heading(text)
        .or_else(|| mnemonic_from_filename(path));

    let Some(mnemonic) = mnemonic else { return; };
    let Ok(instr_type) = InstrType::from_str(&mnemonic) else { return; };

    // Extract summary description.
    let description = extract_summary(text).unwrap_or_default();
    if description.is_empty() { return; }

    // Only update description; preserve all other encoding info from builtin.
    if set.by_type(instr_type).is_some() {
        // Re-fetch to avoid borrow conflict; clone then mutate
        let mut updated = set.by_type(instr_type).unwrap().clone();
        updated.description = description;
        updated.source_file = Some(path.to_string_lossy().into_owned());
        set.insert(updated);
    }
}

/// Extract mnemonic from first `# MNEMONIC X — ...` heading.
fn extract_mnemonic_from_heading(text: &str) -> Option<String> {
    for line in text.lines() {
        let line = line.trim();
        if !line.starts_with("# ") { continue; }
        // "# CCS E — Count, Compare, Skip ..."
        //    ^--- take the word after "# " up to space or end
        let rest = &line[2..].trim();
        let mnemonic = rest.split(|c: char| c.is_whitespace() || c == '—' || c == '-')
            .next()
            .unwrap_or("")
            .trim();
        if !mnemonic.is_empty() {
            return Some(mnemonic.to_uppercase());
        }
    }
    None
}

/// Infer mnemonic from filename stem, e.g. `ccs_e.md` → `"CCS"`.
fn mnemonic_from_filename(path: &Path) -> Option<String> {
    let stem = path.file_stem()?.to_str()?;
    // Take everything before the first underscore.
    let mnemonic = stem.split('_').next()?.to_uppercase();
    Some(mnemonic)
}

/// Extract the first non-empty bullet/line from the Summary section.
fn extract_summary(text: &str) -> Option<String> {
    let mut in_summary = false;
    for line in text.lines() {
        let line = line.trim();

        // Detect the Summary section header
        if line.eq_ignore_ascii_case("summary") || line.eq_ignore_ascii_case("## summary") {
            in_summary = true;
            continue;
        }

        if !in_summary { continue; }

        // Stop at next section header or code block
        if line.starts_with('#') || line.starts_with("```") { break; }

        // Strip markdown list markers and return first non-empty content
        let content = if let Some(rest) = line.strip_prefix("- ") {
            rest.trim()
        } else {
            line
        };

        if !content.is_empty() {
            return Some(content.to_string());
        }
    }
    None
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_whole3_order_code() {
        let (fmt, opc, qtr) = parse_order_code("00.");
        assert_eq!(fmt, OpcodeFormat::Whole3);
        assert_eq!(opc, 0);
        assert_eq!(qtr, None);

        let (fmt, opc, _) = parse_order_code("06.");
        assert_eq!(fmt, OpcodeFormat::Whole3);
        assert_eq!(opc, 6);
    }

    #[test]
    fn parse_quarter5_order_code() {
        let (fmt, opc, qtr) = parse_order_code("01.2");
        assert_eq!(fmt, OpcodeFormat::Quarter5);
        assert_eq!(opc, 1);
        assert_eq!(qtr, Some(2));
    }

    #[test]
    fn parse_channel6_order_code() {
        let (fmt, opc, qtr) = parse_order_code("10.0");
        assert_eq!(fmt, OpcodeFormat::Channel6);
        assert_eq!(opc, 0o10);
        assert_eq!(qtr, Some(0));
    }

    #[test]
    fn parse_special_order_code() {
        let (fmt, _, _) = parse_order_code("00.0006");
        assert_eq!(fmt, OpcodeFormat::Special);
    }

    #[test]
    fn extract_mnemonic_from_h1() {
        let doc = "# CCS E — Count, Compare, Skip (Block-2)\n\nSummary\n- Reads c(E)";
        assert_eq!(extract_mnemonic_from_heading(doc), Some("CCS".to_string()));
    }

    #[test]
    fn extract_summary_from_doc() {
        let doc = "# AD K\n\nSummary\n- Add content of K to register A; set overflow bits.\n\nNotes\n";
        assert_eq!(extract_summary(doc), Some("Add content of K to register A; set overflow bits.".to_string()));
    }

    #[test]
    fn builtin_set_completeness() {
        let set = InstrSpecSet::builtin();
        // Check a sample of expected instructions are present
        assert!(set.by_type(InstrType::Tc).is_some());
        assert!(set.by_type(InstrType::Ca).is_some());
        assert!(set.by_type(InstrType::Ad).is_some());
        assert!(set.by_type(InstrType::Mp).is_some());
        assert!(set.by_type(InstrType::Read).is_some());
        assert!(set.by_type(InstrType::Extend).is_some());
    }
}
