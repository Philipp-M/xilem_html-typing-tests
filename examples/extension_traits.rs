#![allow(unused)]

struct Element;

struct CanvasElement;

trait ElementAttrs {
    fn class(self, class: &str) -> Class<Self>
    where
        Self: std::marker::Sized;
}

trait CanvasElementAttrs {
    type Element;

    fn height(self, height: usize) -> CanvasHeight<Self::Element>;
}

impl ElementAttrs for Element {
    fn class(self, class: &str) -> Class<Self> {
        Class {
            el: self,
            class: class.into(),
        }
    }
}

impl ElementAttrs for CanvasElement {
    fn class(self, class: &str) -> Class<Self> {
        Class {
            el: self,
            class: class.into(),
        }
    }
}

impl CanvasElementAttrs for CanvasElement {
    type Element = CanvasElement;

    fn height(self, height: usize) -> CanvasHeight<Self::Element> {
        CanvasHeight { el: self, height }
    }
}

impl<T: ElementAttrs> ElementAttrs for Class<T> {
    fn class(self, class: &str) -> Class<Self> {
        Class {
            el: self,
            class: class.into(),
        }
    }
}

impl<T: ElementAttrs> ElementAttrs for CanvasHeight<T> {
    fn class(self, class: &str) -> Class<Self> {
        Class {
            el: self,
            class: class.into(),
        }
    }
}

impl<T: CanvasElementAttrs> CanvasElementAttrs for CanvasHeight<T> {
    type Element = Self;

    fn height(self, height: usize) -> CanvasHeight<Self::Element> {
        CanvasHeight { el: self, height }
    }
}

impl<T: CanvasElementAttrs> CanvasElementAttrs for Class<T> {
    type Element = Self;

    fn height(self, height: usize) -> CanvasHeight<Self::Element> {
        CanvasHeight { el: self, height }
    }
}

struct Class<E> {
    el: E,
    class: String,
}

struct CanvasHeight<E> {
    el: E,
    height: usize,
}

fn main() {
    let el = CanvasElement;
    let el = el
        .class("Hello")
        .height(10)
        .class("yet-another-class")
        .height(42);
    println!("sizeof el: {}", std::mem::size_of_val(&el));
    println!("Hello, world!");
}
