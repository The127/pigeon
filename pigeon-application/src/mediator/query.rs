pub trait Query: Send + 'static {
    type Output: Send + 'static;
}
