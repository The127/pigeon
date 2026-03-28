use crate::event::DomainEvent;

pub trait AggregateRoot {
    fn domain_events(&self) -> &[DomainEvent];
    fn clear_events(&mut self);
}
