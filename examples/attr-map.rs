// for convenience below in `IntoClass`
#![feature(return_position_impl_trait_in_trait)]
#![allow(unused)]

use std::collections::btree_map::Entry;
use std::{
    any::Any,
    collections::{BTreeMap, BTreeSet, HashMap},
};

trait View {}

trait ViewSequence {}
impl<T: View> ViewSequence for T {}
impl ViewSequence for () {}
impl<A: ViewSequence, B: ViewSequence> ViewSequence for (A, B) {}
impl<A: ViewSequence, B: ViewSequence, C: ViewSequence> ViewSequence for (A, B, C) {}

impl View for &str {}

macro_rules! define_html_elements {
    ($(($ty_name:ident, $name:ident)),*) => {
        $(
        struct $ty_name<VT> {
            attrs: BTreeMap<&'static str, Box<dyn Any>>,
            children: VT,
        }

        impl<VT> View for $ty_name<VT> {}

        fn $name<VT: ViewSequence>(children: VT) -> $ty_name<VT> {
            $ty_name {
                attrs: Default::default(),
                children,
            }
        }
        )*
    };
}

define_html_elements!((Div, div), (Header, header), (Canvas, canvas), (P, p));

// Since these methods are used for all HTML elements,
// it might make sense to add an extra impl method for even faster compile times (and possibly speed, because of less code-duplication)
macro_rules! impl_element {
    ($( $el:ident),* ) => {
        $(
        impl<VT> $el<VT> {
            fn class<T: IntoClass>(mut self, class: T) -> Self {
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

macro_rules! impl_trivial_attr {
    ($name:ident, $ty: ty, $key: literal) => {
        fn $name(mut self, $name: $ty) -> Self {
            self.attrs
                .entry($key) // TODO: namespacing necessary/useful?
                .and_modify(|attr| {
                    *attr.downcast_mut::<$ty>().unwrap() = $name;
                })
                .or_insert(Box::new($name));
            self
        }
    };
}

macro_rules! impl_canvas_element {
    ($( $el:ident),* ) => {
        $(
        impl<VT> $el<VT> {
            impl_trivial_attr!(height, usize, "canvas_height");
            impl_trivial_attr!(width, usize, "canvas_width");
        }
        )*
    };
}

impl_element!(Div, Canvas, Header, P);

impl_canvas_element!(Canvas);

// A few experiments for more flexible attributes, see in main(): el.class(<IntoClass>)
trait IntoClass {
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
    let _view = div((
        header("Header").class(["header", "bold"]),
        div(p("Hello World!")).class("hello-world"),
        canvas(())
            .width(200)
            .height(100)
            .class(("view", "dynamic".to_string())),
    ));
}
