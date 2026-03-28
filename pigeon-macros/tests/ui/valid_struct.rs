use pigeon_macros::Reconstitute;

#[derive(Reconstitute)]
pub struct Widget {
    id: u64,
    name: String,
    enabled: bool,
}

fn main() {
    let state = WidgetState {
        id: 1,
        name: "test".to_string(),
        enabled: true,
    };

    let widget = Widget::reconstitute(state);

    assert_eq!(widget.id, 1);
    assert_eq!(widget.name, "test");
    assert!(widget.enabled);
}
