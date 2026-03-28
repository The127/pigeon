#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Version(u64);

impl Version {
    pub fn new(value: u64) -> Self {
        Self(value)
    }

    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let v = Version::new(42);
        assert_eq!(v.as_u64(), 42);
    }

    #[test]
    fn equality() {
        assert_eq!(Version::new(1), Version::new(1));
        assert_ne!(Version::new(1), Version::new(2));
    }

    #[test]
    fn copy_semantics() {
        let v1 = Version::new(5);
        let v2 = v1;
        assert_eq!(v1, v2);
    }
}
