use edge_proto::caller::EdgeCaller;

use crate::sys::edge::with_edge_caller_impl;

pub trait EdgeCallerManager<'c>: Sync {
    type Holder: EdgeCaller + 'c;

    fn acquire(&'c self) -> Self::Holder;
}

/// Acquire the edge caller and do something with it (usually issuing edge calls).
pub fn with_edge_caller<F, R>(f: F) -> R
where
    F: FnOnce(&mut dyn EdgeCaller) -> R,
{
    with_edge_caller_impl(f)
}
