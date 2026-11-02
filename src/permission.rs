use core::{fmt, iter, ops};

/// Something a request can be allowed to do to a resource, like pushing to a
/// cache.
//
// Each permission's u8 repr is part of the bytes `Permissions::to_bytes`
// returns, so it must never change, be reused after the permission is removed,
// or be >= 128.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum Permission {
    /// Pulling store paths from a cache.
    PullFromCache = 0,

    /// Pushing store paths to a cache.
    PushToCache = 1,
}

/// A set of [`Permission`]s.
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Permissions(u128);

impl Permission {
    #[inline]
    const fn bit(self) -> u128 {
        1 << self as u8
    }

    #[inline]
    const fn from_repr(repr: u8) -> Option<Self> {
        match repr {
            0 => Some(Self::PullFromCache),
            1 => Some(Self::PushToCache),
            _ => None,
        }
    }
}

impl Permissions {
    /// Returns whether the set contains the given permission.
    #[inline]
    pub const fn contains(self, permission: Permission) -> bool {
        self.0 & permission.bit() != 0
    }

    /// Returns whether the set contains every permission in `other`.
    #[inline]
    pub const fn contains_all(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    /// The inverse of [`to_bytes`](Self::to_bytes).
    ///
    /// Permissions this version of the crate doesn't know are kept when
    /// re-encoding, but [`iter`](Self::iter) skips them.
    #[inline]
    pub const fn from_bytes(bytes: [u8; 16]) -> Self {
        Self(u128::from_be_bytes(bytes))
    }

    /// Returns an iterator over the permissions in the set.
    #[inline]
    pub fn iter(self) -> impl Iterator<Item = Permission> {
        let mut bits = self.0;
        iter::from_fn(move || {
            while bits != 0 {
                let repr = bits.trailing_zeros() as u8;
                // Clears the lowest set bit.
                bits &= bits - 1;
                if let Some(permission) = Permission::from_repr(repr) {
                    return Some(permission);
                }
            }
            None
        })
    }

    /// Creates a set containing the given permissions.
    #[inline]
    pub const fn new<const N: usize>(permissions: [Permission; N]) -> Self {
        let mut bits = 0;
        let mut idx = 0;
        while idx < N {
            bits |= permissions[idx].bit();
            idx += 1;
        }
        Self(bits)
    }

    /// Encodes the set as bytes, e.g. to store it.
    ///
    /// The format is stable, i.e. bytes encoded by one version of this crate
    /// decode to the same set in every later version.
    #[inline]
    pub const fn to_bytes(self) -> [u8; 16] {
        self.0.to_be_bytes()
    }
}

impl fmt::Debug for Permissions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_set().entries(self.iter()).finish()
    }
}

impl FromIterator<Permission> for Permissions {
    #[inline]
    fn from_iter<I: IntoIterator<Item = Permission>>(iter: I) -> Self {
        let mut permissions = Self(0);
        permissions.extend(iter);
        permissions
    }
}

impl Extend<Permission> for Permissions {
    #[inline]
    fn extend<I: IntoIterator<Item = Permission>>(&mut self, iter: I) {
        for permission in iter {
            self.0 |= permission.bit();
        }
    }
}

impl ops::BitOr for Permissions {
    type Output = Self;

    #[inline]
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl ops::BitOrAssign for Permissions {
    #[inline]
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

/// Returns the permissions in `self` that aren't in `rhs`.
impl ops::Sub for Permissions {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self {
        Self(self.0 & !rhs.0)
    }
}
