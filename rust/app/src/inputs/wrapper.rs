use crate::{
    new_wasm_interface::Component,
    util::watchables::{DynWatchableSetter, WatchableSetter},
};
use std::{marker::PhantomData, rc::Rc};

/// Wraps a component in another component
pub trait CompWrapper {
    fn wrap(&self, comp: Component) -> Component;
}
/// Retrieves the input that was wrapped
pub trait ComponentInput: CompWrapper + Clone + 'static {
    type Input;
    type Setter: WatchableSetter<Output = Self::Input> + Clone;
    fn input(&self) -> &Self::Setter;
}
pub trait GetDynWatchableSetter<V> {
    fn dyn_input(&self) -> DynWatchableSetter<V>;
}
impl<I: ComponentInput> GetDynWatchableSetter<I::Input> for I {
    fn dyn_input(&self) -> DynWatchableSetter<I::Input> {
        DynWatchableSetter::new(self.input().clone())
    }
}

/// A trait to carry additional input data
pub trait ComponentInputData: ComponentInput {
    type InputData;
    fn input_data(&self) -> Self::InputData;
}

/// Creates a builder from an input wrapper
pub trait WrapBuilder<I: ComponentInput> {
    type Builder;
    fn builder(wrapper: I) -> Self::Builder;
}

/// The default component type for an input type
pub trait DefaultInputComp: ComponentInput {
    type Comp: WrapBuilder<Self>;
}

// Component input constructors/wrappers
#[derive(Clone)]
pub struct IdentityWrapper;
impl CompWrapper for IdentityWrapper {
    fn wrap(&self, comp: Component) -> Component {
        comp
    }
}

#[derive(Clone)]
pub struct Input<W>(W)
where
    W: WatchableSetter + Clone + 'static;

impl<W> Input<W>
where
    W: WatchableSetter + Clone + 'static,
{
    pub fn new(setter: W) -> Self {
        Self(setter)
    }
}
impl<W> CompWrapper for Input<W>
where
    W: WatchableSetter + Clone + 'static,
{
    fn wrap(&self, comp: Component) -> Component {
        comp
    }
}
impl<W> ComponentInput for Input<W>
where
    W: WatchableSetter + Clone + 'static,
{
    type Input = W::Output;
    type Setter = W;
    fn input(&self) -> &Self::Setter {
        &self.0
    }
}

pub struct InputDefault<W, C>(W, PhantomData<C>)
where
    W: WatchableSetter + Clone + 'static,
    C: WrapBuilder<Self> + 'static;
impl<W, C> Clone for InputDefault<W, C>
where
    W: WatchableSetter + Clone + 'static,
    C: WrapBuilder<Self> + 'static,
{
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1.clone())
    }
}
impl<W, C> InputDefault<W, C>
where
    W: WatchableSetter + Clone + 'static,
    C: WrapBuilder<Self> + 'static,
{
    pub fn new(setter: W) -> Self {
        Self(setter, PhantomData)
    }
}
impl<W, C> CompWrapper for InputDefault<W, C>
where
    W: WatchableSetter + Clone + 'static,
    C: WrapBuilder<Self> + 'static,
{
    fn wrap(&self, comp: Component) -> Component {
        comp
    }
}
impl<W, C> ComponentInput for InputDefault<W, C>
where
    W: WatchableSetter + Clone + 'static,
    C: WrapBuilder<Self> + 'static,
{
    type Input = W::Output;
    type Setter = W;
    fn input(&self) -> &Self::Setter {
        &self.0
    }
}
impl<W, C> DefaultInputComp for InputDefault<W, C>
where
    W: WatchableSetter + Clone + 'static,
    C: WrapBuilder<Self>,
{
    type Comp = C;
}

/// A dynamic component input wrap builder
pub struct DynWrappedInput<F: ComponentInput> {
    map: Rc<dyn Fn(Component) -> Component>,
    wrap_inner: bool,
    input: F,
}
impl<F: ComponentInput> Clone for DynWrappedInput<F> {
    fn clone(&self) -> Self {
        Self {
            map: self.map.clone(),
            wrap_inner: self.wrap_inner.clone(),
            input: self.input.clone(),
        }
    }
}
impl<F: ComponentInput> DynWrappedInput<F> {
    pub fn new<C: Into<Component>, M: Fn(Component) -> C + 'static>(
        input: F,
        map: M,
        wrap_inner: bool,
    ) -> Self {
        Self {
            map: Rc::new(move |comp| (map(comp)).into()),
            wrap_inner,
            input,
        }
    }
}
impl<F: ComponentInput> CompWrapper for DynWrappedInput<F> {
    fn wrap(&self, comp: Component) -> Component {
        match self.wrap_inner {
            true => self.input.wrap((self.map)(comp)),
            false => (self.map)(self.input.wrap(comp)),
        }
    }
}
impl<F: ComponentInput> ComponentInput for DynWrappedInput<F> {
    type Input = F::Input;
    type Setter = F::Setter;
    fn input(&self) -> &Self::Setter {
        self.input.input()
    }
}

impl<F: ComponentInputData> ComponentInputData for DynWrappedInput<F> {
    type InputData = F::InputData;
    fn input_data(&self) -> Self::InputData {
        self.input.input_data()
    }
}
impl<F> DefaultInputComp for DynWrappedInput<F>
where
    F: ComponentInput + DefaultInputComp,
    F::Comp: WrapBuilder<Self>,
{
    type Comp = F::Comp;
}
impl<F> Into<Component> for DynWrappedInput<F>
where
    F: ComponentInput + DefaultInputComp,
    F::Comp: WrapBuilder<Self>,
    <F::Comp as WrapBuilder<Self>>::Builder: Into<Component>,
{
    fn into(self) -> Component {
        F::Comp::builder(self).into()
    }
}
