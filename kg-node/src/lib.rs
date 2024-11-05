pub mod bootstrap;
pub mod ipfs;
pub mod kg;
pub mod ops;
pub mod system_ids;

// Include the `items` module, which is generated from items.proto.
// It is important to maintain the same structure as in the proto.
pub mod grc20 {
    include!(concat!(env!("OUT_DIR"), "/grc20.rs"));
}
