use crate::element_tree::{Element, NoEvent, VirtualDom};
use crate::widgets::EmptySequence;

use crate::element_tree::ReconcileCtx;

use derivative::Derivative;

/// A placeholder element.
///
/// This does **not** represent a blank area or a zero-sized widget. Rather, this represents the *absence* of a widget. So, for instance:
///
/// ```no_compile
/// Column!(
///     Label::new("Hello world!"),
///     EmptyElement::new(),
/// )
/// ```
///
/// is equivalent to:
///
/// ```no_compile
/// Column!(
///     Label::new("Hello world!"),
/// )
/// ```
///
/// in terms of layout computation and everything else.
///
/// ## Events
///
/// Doesn't emit events.
#[derive(Derivative, PartialEq, Eq, Hash)]
#[derivative(Clone(bound = ""), Debug(bound = ""), Default(bound = ""))]
pub struct EmptyElement;

// TODO - Remove Derivative
#[derive(Derivative, PartialEq, Eq, Hash)]
#[derivative(Clone(bound = ""), Debug(bound = ""), Default(bound = ""))]
pub struct EmptyElementData;

impl EmptyElement {
    pub fn new() -> EmptyElement {
        EmptyElement
    }
}

impl Element for EmptyElement {
    type Event = NoEvent;
    type ComponentState = crate::element_tree::NoState;
    type AggregateChildrenState = ();
    type BuildOutput = EmptyElementData;

    fn build(self, _prev_state: ()) -> (EmptyElementData, ()) {
        (EmptyElementData, ())
    }
}

impl VirtualDom for EmptyElementData {
    type Event = NoEvent;
    type AggregateChildrenState = ();
    type TargetWidgetSeq = EmptySequence;

    fn init_tree(&self) -> EmptySequence {
        EmptySequence
    }

    fn reconcile(&self, _other: &Self, _widget_seq: &mut EmptySequence, _ctx: &mut ReconcileCtx) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_debug_snapshot;
    use test_env_log::test;

    #[test]
    fn new_empty() {
        let empty = EmptyElement::new();
        let (empty_data, _) = empty.clone().build(());
        assert_eq!(empty, EmptyElement);
        assert_eq!(empty_data, EmptyElementData);
    }

    #[test]
    fn empty_widget() {
        use crate::test_harness::Harness;

        let empty = EmptyElement::new();

        Harness::run_test_window(empty, |harness| {
            let widget_state = harness.get_root_debug_state();
            assert_debug_snapshot!(widget_state);
        });
    }
}
