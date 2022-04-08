use edge_proto::server::EdgeCallError;

pub trait EdgeErrorCompat<T> {
    fn compat(self) -> anyhow::Result<T>;
}

impl<T> EdgeErrorCompat<T> for Result<T, EdgeCallError> {
    fn compat(self) -> anyhow::Result<T> {
        self.map_err(|_err| anyhow::anyhow!("failed to perform edge call"))
    }
}
