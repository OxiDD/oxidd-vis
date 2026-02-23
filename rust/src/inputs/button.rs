use std::cell::RefCell;

use bon::Builder;

use crate::util::watchables::{
    Constant, DataState, DynWatchable, Field, IntoDynWatchable, IntoWatchable, Listener, Mutator,
    Observer, ReadonlyField, Watchable, WatchableState,
};

/// Button data
pub struct Button(Field<usize>);

impl Button {
    pub fn new() -> Self {
        Button(Field::new(0))
    }
    #[must_use = "Only once the mutator is committed, will the click be performed"]
    pub fn click(&mut self) -> Mutator {
        self.0.set(*self.0.get() + 1)
    }
    pub fn clicks(&self) -> ReadonlyField<usize> {
        self.0.read()
    }
    pub fn on_click<L: FnMut() -> () + 'static>(&self, listener: L) -> Observer {
        self.0.observe(Box::new(ButtonListener::new(listener)))
    }
}
pub struct ButtonListener<L: FnMut() -> () + 'static>(RefCell<L>);
impl<L: FnMut() -> () + 'static> ButtonListener<L> {
    pub fn new(listener: L) -> Self {
        ButtonListener(RefCell::new(listener))
    }
}
impl<L: FnMut() -> () + 'static> Listener for ButtonListener<L> {
    fn state_changed(&self, state: DataState) {
        if state == DataState::UpToDate {
            (self.0.borrow_mut())()
        }
    }
}

// /// Button component
// #[derive(Builder)]
// pub struct ButtonComponent {
//     data: Button,
//     #[builder(with=into_string)]
//     text2: String,
//     #[builder(with=|x: &str| x.to_string())]
//     text3: String,
//     #[builder(with=|val: impl IntoWatchable<String> + 'static| DynWatchable::new(val.into()))]
//     text: DynWatchable<String>,
// }
// pub fn into_dyn_watchable<X, D: IntoDynWatchable<X>>(val: D) -> DynWatchable<X> {
//     val.into()
// }

// pub fn into_string(x: impl Into<String>) -> String {
//     x.into()
// }
// // impl ButtonComponent {
// //     #[builder]
// //     pub fn new(data: Button, text: DynWatchable<String>) -> Self {
// //         Self { data, text }
// //     }
// // }

// fn test() {
//     ButtonComponent::builder().text(Constant::new("test".to_string()));
// }

// #[proc_macro_attribute]
// pub fn into(_attr: TokenStream, item: TokenStream) -> TokenStream {
//     let field = parse_macro_input!(item as Field);

//     let attrs = field.attrs;
//     let vis = field.vis;
//     let ident = field.ident;
//     let ty = field.ty;

//     let expanded = quote! {
//         #[builder(with = |x| x.into())]
//         #(#attrs)*
//         #vis #ident: #ty
//     };

//     TokenStream::from(expanded)
// }
