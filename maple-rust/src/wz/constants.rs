//! Contains all the constant values used for various functions in the WZ file encryption.

/// IV used to create the WzKey for GMS
pub const WZ_GMSIV: [u8; 4] = [0x4D, 0x23, 0xC7, 0x2B];

/// IV used to create the WzKey for the latest version of GMS, MSEA, or KMS
pub const WZ_MSEAIV: [u8; 4] = [0xB9, 0x7D, 0x63, 0xE9];

/// IV used to create the WzKey for the latest version of GMS, MSEA, or KMS
pub const WZ_BMSCLASSIC: [u8; 4] = [0x00, 0x00, 0x00, 0x00];

/// Constant used in WZ offset encryption
pub const WZ_OFFSET_CONSTANT: u32 = 0x581C3F6D;