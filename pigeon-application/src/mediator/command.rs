pub trait Command: Send + 'static {
    type Output: Send + 'static;
    fn command_name(&self) -> &'static str;
}
