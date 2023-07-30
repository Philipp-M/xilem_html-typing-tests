// for convenience below in `IntoClass`
#![feature(return_position_impl_trait_in_trait)]

use std::collections::btree_map::Entry;
use std::{
    any::Any,
    collections::{BTreeMap, BTreeSet},
};

pub trait View {}

pub trait ViewSequence {}
impl<T: View> ViewSequence for T {}
impl ViewSequence for () {}
impl<A: ViewSequence, B: ViewSequence> ViewSequence for (A, B) {}
impl<A: ViewSequence, B: ViewSequence, C: ViewSequence> ViewSequence for (A, B, C) {}

impl View for &str {}

type Attrs = BTreeMap<&'static str, Box<dyn Any>>;

macro_rules! define_html_elements {
    ($(($ty_name:ident, $name:ident)),*) => {
        $(
        pub struct $ty_name<VT> {
            pub(crate) attrs: Attrs,
            #[allow(unused)]
            children: VT,
        }

        impl<VT> View for $ty_name<VT> {}

        pub fn $name<VT: ViewSequence>(children: VT) -> $ty_name<VT> {
            $ty_name {
                attrs: Default::default(),
                children,
            }
        }

        impl<VT> WithAttrs for $ty_name<VT> {
            fn raw_attrs(&self) -> &Attrs {
                &self.attrs
            }
            fn raw_attrs_mut(&mut self) -> &mut Attrs {
                &mut self.attrs
            }
        }
        )*
    };
}

// separate module to avoid naming collisions with the respective traits
// TODO decision of naming concrete elements vs traits?
// (Maybe just use DOM interface names for the traits as below, and keep the names of the concrete views simple as shown here)
mod elements {
    use super::{Attrs, View, ViewSequence, WithAttrs};
    define_html_elements!((Div, div), (Header, header), (Canvas, canvas), (P, p));
}

pub trait WithAttrs {
    fn raw_attrs(&self) -> &Attrs;
    // TODO allow mutable attributes?
    fn raw_attrs_mut(&mut self) -> &mut Attrs;
}

/// These traits should mirror the respective DOM interfaces
/// In this case https://dom.spec.whatwg.org/#interface-element
/// Or rather a curated/opinionated subset that makes sense in xilem for each of these interfaces
/// unfortunately with this (builder) pattern not trait-object-safe
pub trait Element: WithAttrs {
    fn class<T: IntoClass>(self, class: T) -> Self;
}

pub trait HTMLElement: Element {}

// not sure if an extra trait for this makes sense, but for consistency
pub trait HTMLCanvasElement: HTMLElement {
    fn width(self, width: usize) -> Self;
    fn height(self, height: usize) -> Self;
}

fn impl_simple_diff_attr<T: PartialEq + 'static>(
    a_attrs: &Attrs,
    b_attrs: &Attrs,
    key: &'static str,
) -> bool {
    match (a_attrs.get(key), b_attrs.get(key)) {
        (None, None) => false,
        (None, Some(_)) | (Some(_), None) => true,
        (Some(a), Some(b)) => a.downcast_ref::<T>().unwrap() == b.downcast_ref::<T>().unwrap(),
    }
}

// TODO diff untyped attributes (escape hatch for e.g. webcomponents or custom attributes (e.g. `data-...`))
// One idea is to have a reserved key directly in the attrs itself showing (BTreeSet<&str>)
// whereas the values are likely strings as in the current xilem_html implementation

// The following diffing functions could also return `Attrs` (BTreeSet) containing the diff (or maybe an Option<Attrs> or an enum with more information what was added/deleted)

/// returns whether attributes belonging to the Element interface are different (currently just `class`)
// TODO include custom "untyped" attributes
#[allow(unused)]
// TODO inline? maybe create another inner impl function that sums up all diffs of various attributes and inline this one?
#[inline(always)]
pub fn element_diff<T: Element>(a: &T, b: &T) -> bool {
    impl_simple_diff_attr::<BTreeSet<String>>(a.raw_attrs(), b.raw_attrs(), "class")
}

#[allow(unused)]
#[inline(always)]
pub fn html_element_diff<T: HTMLElement>(a: &T, b: &T) -> bool {
    element_diff(a, b)
}

/// returns whether attributes belonging to the HTMLCanvasElement interface are different
#[allow(unused)]
#[inline(always)]
pub fn canvas_element_diff<T: HTMLCanvasElement>(a: &T, b: &T) -> bool {
    html_element_diff(a, b)
        || impl_simple_diff_attr::<usize>(a.raw_attrs(), b.raw_attrs(), "canvas_width")
        || impl_simple_diff_attr::<usize>(a.raw_attrs(), b.raw_attrs(), "canvas_height")
}

// Since these methods are used for all HTML elements,
// it might make sense to add an extra inner impl function if possible (see below at `simple_attr_impl` for an example)
macro_rules! impl_element {
    ($( $el:ident),* ) => {
        $(
        impl<VT> Element for elements::$el<VT> {
            fn class<T: IntoClass>(mut self, class: T) -> elements::$el<VT> {
                match self.attrs.entry("class") {
                    Entry::Vacant(entry) => {
                        entry.insert(Box::new(BTreeSet::from_iter(class.classes())));
                    },
                    Entry::Occupied(class_attr) => {
                        let class_attr = class_attr.into_mut().downcast_mut::<BTreeSet<String>>().unwrap();
                        for class in class.classes() {
                            class_attr.insert(class);
                        }
                    },
                };
                self
            }
        }
        )*
    };
}

macro_rules! impl_html_element {
    ($( $el:ident),* ) => { $(impl<VT> HTMLElement for elements::$el<VT> {})* }
}

macro_rules! impl_canvas_element {
    ($( $el:ident),* ) => {
        $(
        impl<VT> HTMLCanvasElement for elements::$el<VT> {
            // TODO are namespaces (here "canvas_") useful at all (especially with custom attributes in mind)?
            impl_simple_attr!(width, usize, "canvas_width", elements::$el<VT>);
            impl_simple_attr!(height, usize, "canvas_height", elements::$el<VT>);
        }
        )*
    };
}

macro_rules! impl_simple_attr {
    ($name:ident, $ty: ty, $key: literal, $el: ty) => {
        #[inline(always)]
        fn $name(mut self, $name: $ty) -> $el {
            simple_attr_impl(&mut self.attrs, $key, $name);
            self
        }
    };
}

fn simple_attr_impl<T: 'static>(
    attrs: &mut BTreeMap<&'static str, Box<dyn Any>>,
    key: &'static str,
    value: T,
) {
    match attrs.entry(key) {
        Entry::Vacant(entry) => {
            entry.insert(Box::new(value));
        }
        Entry::Occupied(class_attr) => {
            *class_attr.into_mut().downcast_mut::<T>().unwrap() = value;
        }
    };
}

impl_element!(Div, Canvas, Header, P);

impl_html_element!(Div, Canvas, Header, P);

impl_canvas_element!(Canvas);

// A few experiments for more flexible attributes, see in main(): el.class(<IntoClass>)
pub trait IntoClass {
    fn classes(self) -> impl Iterator<Item = String>;
}

impl IntoClass for &str {
    fn classes(self) -> impl Iterator<Item = String> {
        Some(self.into()).into_iter()
    }
}

impl IntoClass for String {
    fn classes(self) -> impl Iterator<Item = String> {
        Some(self).into_iter()
    }
}

impl<T: IntoClass, const N: usize> IntoClass for [T; N] {
    fn classes(self) -> impl Iterator<Item = String> {
        self.into_iter().flat_map(IntoClass::classes)
    }
}

// TODO do we want to use the tuple syntax here ("conflicts" with ViewSequence)?
// It allows different types for each tuple member though, which might be useful,
// but an alternative would be multiple class invocations with different types
impl<A: IntoClass, B: IntoClass> IntoClass for (A, B) {
    fn classes(self) -> impl Iterator<Item = String> {
        self.0.classes().chain(self.1.classes())
    }
}

fn main() {
    use elements::*;
    let _view = div((
        header("Header").class(["header", "bold"]),
        div(p("Hello World!")).class("hello-world"),
        // composition instead of builder pattern
        // if Element::class() is too verbose when using composition, one could use something like this:
        // fn class<T: Element, C: IntoClass>(el: T, class: C) -> T {
        //     el.class(class)
        // }
        Element::class(canvas(()), ["game", "render-view"])
            .width(200)
            .height(100)
            .class(("view", "dynamic".to_string())),
    ));
}
