use crate::{ids::Grc20Id, pb::ipfs};

/// A trait for converting a type to a sequence of triples.
pub trait ToTriples {
    fn to_triples(&self) -> impl Iterator<Item = ipfs::Triple>;
}

/// A trait for creating a type from a sequence of triples.
pub trait FromTriples: Sized {
    type Error;

    fn from_triples(
        id: Grc20Id,
        triples: impl IntoIterator<Item = ipfs::Triple>,
    ) -> Result<Self, Self::Error>;
}

pub trait ToOps {
    fn to_ops(&self) -> impl Iterator<Item = ipfs::Op>;
}

pub trait FromOps: Sized {
    type Error;

    fn from_ops(id: Grc20Id, ops: impl IntoIterator<Item = ipfs::Op>) -> Result<Self, Self::Error>;
}
