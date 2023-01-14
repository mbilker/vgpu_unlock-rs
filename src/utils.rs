use std::fmt;

#[cfg(feature = "proxmox")]
use crate::uuid::Uuid;

#[derive(Clone, Copy)]
#[repr(C, align(8))]
pub struct AlignedU64(pub u64);

impl fmt::Debug for AlignedU64 {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

impl fmt::LowerHex for AlignedU64 {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::LowerHex::fmt(&self.0, f)
    }
}

/// Extracts the VMID from the last segment of a mdev uuid
///
/// For example, for this uuid 00000000-0000-0000-0000-000000000100
/// it would extract the number 100
///
/// All except the last segment must be zero
#[cfg(feature = "proxmox")]
pub fn uuid_to_vmid(uuid: Uuid) -> Option<u64> {
    // Ensure that the first parts of the uuid are only 0
    if uuid.0 != 0 || uuid.1 != 0 || uuid.2 != 0 || uuid.3[0] != 0 || uuid.3[1] != 0 {
        return None;
    }

    // Format the last segment of the uuid
    let s = format!(
        "{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        uuid.3[2], uuid.3[3], uuid.3[4], uuid.3[5], uuid.3[6], uuid.3[7]
    );

    // Parse it as a normal decimal number to get the right vm id
    s.parse().ok()
}
