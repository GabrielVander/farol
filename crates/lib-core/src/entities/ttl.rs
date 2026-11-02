/// A record's time to live, in seconds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ttl(u32);

impl Ttl {
    pub fn seconds(&self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TtlError {
    Zero,
}

impl TryFrom<u32> for Ttl {
    type Error = TtlError;

    fn try_from(seconds: u32) -> Result<Self, Self::Error> {
        match seconds {
            0 => Err(TtlError::Zero),
            _ => Ok(Self(seconds)),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::entities::ttl::{Ttl, TtlError};

    #[test]
    fn should_reject_zero() {
        assert_eq!(Err(TtlError::Zero), Ttl::try_from(0));
    }

    #[test]
    fn should_accept_a_positive_number_of_seconds() {
        let ttl = Ttl::try_from(300).expect("valid ttl");

        assert_eq!(300, ttl.seconds());
    }
}
