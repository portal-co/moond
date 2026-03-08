//! `InstrType` enum — every distinct AGC Block-2 instruction variant.

use core::fmt;
use core::str::FromStr;

/// Every distinct AGC Block-2 instruction.
///
/// Variant names match the canonical AGC mnemonics (PascalCase).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum InstrType {
    // Sequence-changing
    Tc,    // Transfer Control to K
    Tcf,   // Transfer Control to Fixed F
    Ccs,   // Count, Compare, and Skip on E  (extracode)
    Bzf,   // Branch on Zero to Fixed F
    Bzmf,  // Branch on Zero or Minus to Fixed F

    // Fetching and storing
    Ca,    // Clear and Add K
    Cs,    // Clear and Subtract K
    Dca,   // Double Clear and Add K        (extracode)
    Dcs,   // Double Clear and Subtract K   (extracode)
    Ts,    // Transfer to Storage E         (extracode)
    Xch,   // Exchange A and E              (extracode)
    Lxch,  // Exchange L and E              (extracode)
    Qxch,  // Exchange Q and E              (extracode)
    Dxch,  // Double Exchange A and E       (extracode)

    // Modifying
    Ndx,   // Index (basic K-form or extracode E-form)

    // Arithmetic and logic
    Ad,    // Add K
    Su,    // Subtract E                    (extracode)
    Mp,    // Multiply by K                 (extracode)
    Dv,    // Divide by E                   (extracode)
    Ads,   // Add to Storage E              (extracode)
    Das,   // Double Add to Storage E       (extracode)
    Incr,  // Increment E                   (extracode)
    Aug,   // Augment E                     (extracode)
    Dim,   // Diminish E                    (extracode)
    Msu,   // Modular Subtract E            (extracode)
    Msk,   // Mask with K

    // I/O channels
    Read,  // Read channel H into A
    Write, // Write A to channel H
    Rand,  // Read channel AND A → A
    Wand,  // Write A AND channel → channel (extracode)
    Ror,   // Read channel OR A → A
    Wor,   // Write A OR channel → channel  (extracode)
    Rxor,  // Read channel XOR A → A        (extracode)

    // Special / control
    Extend, // Enable extracode for next instruction
    Inhint, // Inhibit interrupt
    Relint, // Release interrupt inhibit
    Resume, // Resume interrupted program
    Go,     // Restart at fixed address 04000

    // Involuntary — interrupt
    Rupt,   // Interrupt (saves state, vectors to handler)

    // Counter instructions (hardware-triggered, no opcode bits)
    Pinc,  // Plus Increment C
    Minc,  // Minus Increment C
    Dinc,  // Diminish Increment C
    Pcdu,  // Plus Counter Down-Up C
    Mcdu,  // Minus Counter Down-Up C
    Shinc, // Shift Increment C
    Shanc, // Shift and Add Increment C

    // Peripheral / GSE
    Tcsaj,  // Transfer Control to Specified Address K
    Fetch,  // Fetch K (display on GSE)
    Store,  // Store E (load from GSE)      (extracode)
    Inotrd, // I/O Not Read H (display on GSE)
    Inotld, // I/O Not Load H (load from GSE)

    Unknown,
}

impl InstrType {
    /// Return the canonical mnemonic string.
    pub fn mnemonic(self) -> &'static str {
        match self {
            InstrType::Tc     => "TC",
            InstrType::Tcf    => "TCF",
            InstrType::Ccs    => "CCS",
            InstrType::Bzf    => "BZF",
            InstrType::Bzmf   => "BZMF",
            InstrType::Ca     => "CA",
            InstrType::Cs     => "CS",
            InstrType::Dca    => "DCA",
            InstrType::Dcs    => "DCS",
            InstrType::Ts     => "TS",
            InstrType::Xch    => "XCH",
            InstrType::Lxch   => "LXCH",
            InstrType::Qxch   => "QXCH",
            InstrType::Dxch   => "DXCH",
            InstrType::Ndx    => "NDX",
            InstrType::Ad     => "AD",
            InstrType::Su     => "SU",
            InstrType::Mp     => "MP",
            InstrType::Dv     => "DV",
            InstrType::Ads    => "ADS",
            InstrType::Das    => "DAS",
            InstrType::Incr   => "INCR",
            InstrType::Aug    => "AUG",
            InstrType::Dim    => "DIM",
            InstrType::Msu    => "MSU",
            InstrType::Msk    => "MSK",
            InstrType::Read   => "READ",
            InstrType::Write  => "WRITE",
            InstrType::Rand   => "RAND",
            InstrType::Wand   => "WAND",
            InstrType::Ror    => "ROR",
            InstrType::Wor    => "WOR",
            InstrType::Rxor   => "RXOR",
            InstrType::Extend => "EXTEND",
            InstrType::Inhint => "INHINT",
            InstrType::Relint => "RELINT",
            InstrType::Resume => "RESUME",
            InstrType::Go     => "GO",
            InstrType::Rupt   => "RUPT",
            InstrType::Pinc   => "PINC",
            InstrType::Minc   => "MINC",
            InstrType::Dinc   => "DINC",
            InstrType::Pcdu   => "PCDU",
            InstrType::Mcdu   => "MCDU",
            InstrType::Shinc  => "SHINC",
            InstrType::Shanc  => "SHANC",
            InstrType::Tcsaj  => "TCSAJ",
            InstrType::Fetch  => "FETCH",
            InstrType::Store  => "STORE",
            InstrType::Inotrd => "INOTRD",
            InstrType::Inotld => "INOTLD",
            InstrType::Unknown => "UNKNOWN",
        }
    }

    /// True if this instruction normally requires an EXTEND prefix to decode.
    pub fn requires_extend(self) -> bool {
        matches!(self,
            InstrType::Ccs  | InstrType::Dca  | InstrType::Dcs  |
            InstrType::Ts   | InstrType::Xch  | InstrType::Lxch |
            InstrType::Qxch | InstrType::Dxch | InstrType::Su   |
            InstrType::Mp   | InstrType::Dv   | InstrType::Ads  |
            InstrType::Das  | InstrType::Incr | InstrType::Aug  |
            InstrType::Dim  | InstrType::Msu  | InstrType::Wand |
            InstrType::Wor  | InstrType::Rxor | InstrType::Store
        )
    }
}

impl FromStr for InstrType {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, ()> {
        // Manual uppercase comparison without std::string
        let bytes = s.trim().as_bytes();
        // Build uppercase version in a fixed-size buffer
        let mut buf = [0u8; 8];
        if bytes.len() > buf.len() { return Err(()); }
        for (i, &b) in bytes.iter().enumerate() {
            buf[i] = b.to_ascii_uppercase();
        }
        let upper = core::str::from_utf8(&buf[..bytes.len()]).map_err(|_| ())?;
        match upper {
            "TC"     => Ok(InstrType::Tc),
            "TCF"    => Ok(InstrType::Tcf),
            "CCS"    => Ok(InstrType::Ccs),
            "BZF"    => Ok(InstrType::Bzf),
            "BZMF"   => Ok(InstrType::Bzmf),
            "CA"     => Ok(InstrType::Ca),
            "CS"     => Ok(InstrType::Cs),
            "DCA"    => Ok(InstrType::Dca),
            "DCS"    => Ok(InstrType::Dcs),
            "TS"     => Ok(InstrType::Ts),
            "XCH"    => Ok(InstrType::Xch),
            "LXCH"   => Ok(InstrType::Lxch),
            "QXCH"   => Ok(InstrType::Qxch),
            "DXCH"   => Ok(InstrType::Dxch),
            "NDX"    => Ok(InstrType::Ndx),
            "AD"     => Ok(InstrType::Ad),
            "SU"     => Ok(InstrType::Su),
            "MP"     => Ok(InstrType::Mp),
            "DV"     => Ok(InstrType::Dv),
            "ADS"    => Ok(InstrType::Ads),
            "DAS"    => Ok(InstrType::Das),
            "INCR"   => Ok(InstrType::Incr),
            "AUG"    => Ok(InstrType::Aug),
            "DIM"    => Ok(InstrType::Dim),
            "MSU"    => Ok(InstrType::Msu),
            "MSK"    => Ok(InstrType::Msk),
            "READ"   => Ok(InstrType::Read),
            "WRITE"  => Ok(InstrType::Write),
            "RAND"   => Ok(InstrType::Rand),
            "WAND"   => Ok(InstrType::Wand),
            "ROR"    => Ok(InstrType::Ror),
            "WOR"    => Ok(InstrType::Wor),
            "RXOR"   => Ok(InstrType::Rxor),
            "EXTEND" => Ok(InstrType::Extend),
            "INHINT" => Ok(InstrType::Inhint),
            "RELINT" => Ok(InstrType::Relint),
            "RESUME" => Ok(InstrType::Resume),
            "GO"     => Ok(InstrType::Go),
            "RUPT"   => Ok(InstrType::Rupt),
            "PINC"   => Ok(InstrType::Pinc),
            "MINC"   => Ok(InstrType::Minc),
            "DINC"   => Ok(InstrType::Dinc),
            "PCDU"   => Ok(InstrType::Pcdu),
            "MCDU"   => Ok(InstrType::Mcdu),
            "SHINC"  => Ok(InstrType::Shinc),
            "SHANC"  => Ok(InstrType::Shanc),
            "TCSAJ"  => Ok(InstrType::Tcsaj),
            "FETCH"  => Ok(InstrType::Fetch),
            "STORE"  => Ok(InstrType::Store),
            "INOTRD" => Ok(InstrType::Inotrd),
            "INOTLD" => Ok(InstrType::Inotld),
            _        => Err(()),
        }
    }
}

impl fmt::Display for InstrType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.mnemonic())
    }
}
