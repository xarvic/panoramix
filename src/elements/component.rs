use crate::element_tree::{CompCtx, ProcessEventCtx, ReconcileCtx};
use crate::element_tree::{Element, Metadata, NoState, VirtualDom};
use crate::elements::ElementBox;
use crate::glue::GlobalEventCx;

use derivative::Derivative;
use std::fmt::Debug;

pub trait Component: Debug + Clone {
    type Props: Clone + Default + Debug + PartialEq + 'static;
    type LocalEvent: Clone + Debug + PartialEq + 'static;
    type LocalState: Clone + Default + Debug + PartialEq + 'static;

    fn new(props: Self::Props) -> ElementBox<Self::LocalEvent>;

    fn name() -> &'static str;
}

#[derive(Derivative, Default, PartialEq, Eq, Hash)]
#[derivative(Clone(bound = "Comp::Props: Clone"))]
pub struct ComponentHolder<
    Comp: Component,
    ReturnedTree: Element<Event = Comp::LocalEvent>,
    CompFn: Clone + Fn(&CompCtx, Comp::Props) -> ReturnedTree,
> {
    component_fn: CompFn,
    props: Comp::Props,
    _marker: std::marker::PhantomData<Comp>,
}

#[derive(Derivative, Hash)]
#[derivative(
    Clone(bound = ""),
    Default(bound = "Child: Default"),
    PartialEq(bound = "Child: PartialEq"),
    Eq(bound = "Child: Eq")
)]
pub struct ComponentOutput<
    ComponentEvent: Clone + Debug + PartialEq,
    ComponentState: Clone + Default + Debug + PartialEq,
    Child: Element,
> {
    pub child: Child,
    pub name: &'static str,
    #[derivative(Debug = "ignore")]
    pub _metadata: Metadata<ComponentEvent, ComponentState>,
}

#[derive(Derivative, Hash)]
#[derivative(
    Clone(bound = "Child: Clone"),
    Default(bound = "Child: Default"),
    PartialEq(bound = "Child: PartialEq"),
    Eq(bound = "Child: Eq")
)]
pub struct ComponentOutputData<
    ComponentEvent: Clone + Debug + PartialEq,
    ComponentState: Clone + Default + Debug + PartialEq,
    Child: VirtualDom,
> {
    pub child: Child,
    pub name: &'static str,
    #[derivative(Debug = "ignore")]
    pub _metadata: Metadata<ComponentEvent, ComponentState>,
}

// ---

impl<
        Comp: Component,
        ReturnedTree: Element<Event = Comp::LocalEvent>,
        CompFn: Clone + Fn(&CompCtx, Comp::Props) -> ReturnedTree,
    > ComponentHolder<Comp, ReturnedTree, CompFn>
{
    pub fn new(component_fn: CompFn, props: Comp::Props) -> Self {
        Self {
            component_fn,
            props,
            _marker: Default::default(),
        }
    }
}

impl<
        Comp: Component,
        ReturnedTree: Element<Event = Comp::LocalEvent>,
        CompFn: Clone + Fn(&CompCtx, Comp::Props) -> ReturnedTree,
    > std::fmt::Debug for ComponentHolder<Comp, ReturnedTree, CompFn>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple(Comp::name()).field(&self.props).finish()
    }
}

impl<
        ComponentEvent: Clone + Debug + PartialEq + 'static,
        ComponentState: Clone + Default + Debug + PartialEq + 'static,
        Child: Element,
    > std::fmt::Debug for ComponentOutput<ComponentEvent, ComponentState, Child>
{
    #[rustfmt::skip]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple(self.name)
            .field(&self.child)
        .finish()
    }
}

impl<
        ComponentEvent: Clone + Debug + PartialEq + 'static,
        ComponentState: Clone + Default + Debug + PartialEq + 'static,
        Child: VirtualDom,
    > std::fmt::Debug for ComponentOutputData<ComponentEvent, ComponentState, Child>
{
    #[rustfmt::skip]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple(self.name)
            .field(&self.child)
        .finish()
    }
}

// ---

impl<
        Comp: Component,
        ReturnedTree: Element<Event = Comp::LocalEvent>,
        CompFn: Clone + Fn(&CompCtx, Comp::Props) -> ReturnedTree,
    > Element for ComponentHolder<Comp, ReturnedTree, CompFn>
{
    type Event = Comp::LocalEvent;
    type ComponentState = NoState;
    type AggregateChildrenState = ReturnedTree::AggregateChildrenState;
    type BuildOutput = ReturnedTree::BuildOutput;

    // TODO - add spans
    fn build(
        self,
        prev_state: Self::AggregateChildrenState,
    ) -> (Self::BuildOutput, Self::AggregateChildrenState) {
        let default_state = Default::default();
        let local_state = ReturnedTree::get_component_state(&prev_state).unwrap_or(&default_state);

        let ctx = CompCtx {
            local_state: local_state,
        };
        let element_tree = (self.component_fn)(&ctx, self.props);

        element_tree.build(prev_state)
    }
}

/// ---

impl<
        ComponentEvent: Clone + Debug + PartialEq + 'static,
        ComponentState: Clone + Default + Debug + PartialEq + 'static,
        Child: Element,
    > Element for ComponentOutput<ComponentEvent, ComponentState, Child>
{
    type Event = ComponentEvent;

    type ComponentState = ComponentState;
    // TODO - Store Event queue somewhere else?
    type AggregateChildrenState = (
        Vec<ComponentEvent>,
        ComponentState,
        Child::AggregateChildrenState,
    );
    type BuildOutput = ComponentOutputData<ComponentEvent, ComponentState, Child::BuildOutput>;

    fn build(
        self,
        prev_state: Self::AggregateChildrenState,
    ) -> (Self::BuildOutput, Self::AggregateChildrenState) {
        let (_, prev_local_state, children_prev_state) = prev_state;
        let (child, children_state) = self.child.build(children_prev_state);
        (
            ComponentOutputData {
                child,
                name: self.name,
                _metadata: Default::default(),
            },
            (vec![], prev_local_state, children_state),
        )
    }

    fn get_component_state(state: &Self::AggregateChildrenState) -> Option<&Self::ComponentState> {
        Some(&state.1)
    }
}

impl<
        ComponentEvent: Clone + Debug + PartialEq + 'static,
        ComponentState: Clone + Default + Debug + PartialEq + 'static,
        Child: VirtualDom,
    > VirtualDom for ComponentOutputData<ComponentEvent, ComponentState, Child>
{
    type Event = ComponentEvent;
    type AggregateChildrenState = (
        Vec<ComponentEvent>,
        ComponentState,
        Child::AggregateChildrenState,
    );
    type TargetWidgetSeq = Child::TargetWidgetSeq;

    // TODO - add spans
    fn init_tree(&self) -> Child::TargetWidgetSeq {
        self.child.init_tree()
    }

    fn reconcile(
        &self,
        other: &Self,
        widget_seq: &mut Child::TargetWidgetSeq,
        ctx: &mut ReconcileCtx,
    ) {
        self.child.reconcile(&other.child, widget_seq, ctx);
    }

    fn process_local_event(
        &self,
        children_state: &mut Self::AggregateChildrenState,
        _widget_seq: &mut Child::TargetWidgetSeq,
        _cx: &mut GlobalEventCx,
    ) -> Option<Self::Event> {
        let event_queue = &mut children_state.0;
        // TODO - this is a stack, not a queue; whatever, I'll use VecDeque later
        event_queue.pop()
    }

    fn process_event(
        &self,
        _comp_ctx: &mut ProcessEventCtx,
        children_state: &mut Self::AggregateChildrenState,
        widget_seq: &mut Self::TargetWidgetSeq,
        cx: &mut GlobalEventCx,
    ) {
        let mut ctx = ProcessEventCtx {
            event_queue: &mut children_state.0,
            state: &mut children_state.1,
        };
        self.child
            .process_event(&mut ctx, &mut children_state.2, widget_seq, cx)
    }
}

#[cfg(test)]
mod tests {
    #![allow(dead_code)]

    use crate as panoramix;

    #[derive(Debug, Default, Clone, PartialEq, Hash)]
    struct MyComponent;

    type MyPropsType = ();
    type MyLocalEvent = panoramix::NoEvent;
    type MyLocalState = u16;

    impl MyComponent {
        fn new(props: MyPropsType) -> impl panoramix::Element<Event = MyLocalEvent> {
            <Self as panoramix::elements::component::Component>::new(props)
        }

        fn render(
            _ctx: &panoramix::CompCtx,
            _my_props: MyPropsType,
        ) -> impl panoramix::Element<Event = MyLocalEvent> {
            let child = { panoramix::elements::EmptyElement::new() };
            panoramix::elements::component::ComponentOutput {
                child,
                name: "MyComponent",
                _metadata: panoramix::backend::Metadata::<MyLocalEvent, MyLocalState>::new(),
            }
        }
    }

    impl panoramix::elements::component::Component for MyComponent {
        type Props = MyPropsType;
        type LocalState = MyLocalState;
        type LocalEvent = MyLocalEvent;

        fn new(props: Self::Props) -> panoramix::elements::ElementBox<MyLocalEvent> {
            panoramix::elements::ElementBox::new(panoramix::elements::backend::ComponentHolder::<
                Self,
                _,
                _,
            >::new(&MyComponent::render, props))
        }

        fn name() -> &'static str {
            "MyComponent"
        }
    }

    use crate::element_tree::assign_empty_state_type;
    use crate::element_tree::Element;
    use insta::assert_debug_snapshot;
    use test_env_log::test;

    #[test]
    fn call_component() {
        let my_component = MyComponent::new(());
        assign_empty_state_type(&my_component);

        let (component_result, _state) = my_component.build(Default::default());
        assert_debug_snapshot!(component_result);

        //let prev_state = (999, Default::default());
        //let (component_result, component_state) = my_component.build(prev_state);
        //assert_eq!(component_state.0, 999);

        // TODO - local state
        // TODO - process_event
    }

    // TODO
    // - Widget test
    // - Events
}
