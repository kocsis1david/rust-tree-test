use std::{
    ops::{Deref, DerefMut},
    rc::{Rc, Weak},
};

use clone_cell::cell::Cell;

pub enum AttachTarget {
    FirstChild,
    LastChild,
    After,
    Before,
}

pub struct Node<T> {
    // `Node<T>` behaves like it has strong references to its children, but it updates the strong
    // count to its children with `Rc::increment_strong_count` instead of having a
    // children: Vec<Rc<Node<T>>> field
    parent: Cell<Weak<Node<T>>>,
    first_child: Cell<Weak<Node<T>>>,
    last_child: Cell<Weak<Node<T>>>,
    prev_sibling: Cell<Weak<Node<T>>>,
    next_sibling: Cell<Weak<Node<T>>>,

    value: T,
}

impl<T> Node<T> {
    pub fn new(value: T) -> Rc<Node<T>> {
        Rc::new(Node {
            parent: Default::default(),
            first_child: Default::default(),
            last_child: Default::default(),
            prev_sibling: Default::default(),
            next_sibling: Default::default(),
            value,
        })
    }

    pub fn parent(&self) -> Weak<Self> {
        self.parent.get()
    }

    pub fn first_child(&self) -> Weak<Self> {
        self.first_child.get()
    }

    pub fn last_child(&self) -> Weak<Self> {
        self.prev_sibling.get()
    }

    pub fn prev_sibling(&self) -> Weak<Self> {
        self.prev_sibling.get()
    }

    pub fn next_sibling(&self) -> Weak<Self> {
        self.next_sibling.get()
    }

    pub fn is_root(&self) -> bool {
        let parent = self.parent.take();
        let ret = parent.ptr_eq(&Weak::new());
        self.parent.set(parent);
        ret
    }

    pub fn add_child_last(self: &Rc<Self>, node: &Rc<Self>) {
        self.attach(AttachTarget::LastChild, node);
    }

    pub fn remove_last_child(&self) -> Option<Rc<Self>> {
        self.last_child().upgrade().map(|x| {
            x.detach();
            x
        })
    }

    pub fn detach(self: &Rc<Self>) {
        let parent = self.parent.take();
        let prev_sibling = self.prev_sibling.take();
        let next_sibling = self.next_sibling.take();

        let Some(parent) = parent.upgrade()
        else {
            return;
        };

        unsafe {
            Rc::decrement_strong_count(Rc::as_ptr(self));
        }

        if let Some(prev_sibling) = prev_sibling.upgrade() {
            prev_sibling.next_sibling.set(next_sibling.clone());
        } else {
            parent.first_child.set(next_sibling.clone());
        }

        if let Some(next_sibling) = next_sibling.upgrade() {
            next_sibling.prev_sibling.set(prev_sibling);
        } else {
            parent.last_child.set(prev_sibling);
        }
    }

    pub fn attach(self: &Rc<Self>, attach_target: AttachTarget, node: &Rc<Self>) {
        assert!(node.is_root());

        match attach_target {
            AttachTarget::Before => {
                let parent = self.parent.get().upgrade();
                let parent = parent.expect("Cannot attach node as sibling to a root node");
                let prev = self.prev_sibling.get().upgrade();
                _attach(parent, prev, Some(self.clone()), node);
            }
            AttachTarget::After => {
                let parent = self.parent.get().upgrade();
                let parent = parent.expect("Cannot attach node as sibling to a root node");
                let next = self.next_sibling.get().upgrade();
                _attach(parent, Some(self.clone()), next, node);
            }
            AttachTarget::FirstChild => {
                let next = self.first_child.get().upgrade();
                _attach(self.clone(), None, next, node);
            }
            AttachTarget::LastChild => {
                let prev = self.last_child.get().upgrade();
                _attach(self.clone(), prev, None, node);
            }
        }

        unsafe {
            Rc::increment_strong_count(Rc::as_ptr(node));
        }

        fn _attach<T>(
            parent: Rc<Node<T>>,
            prev: Option<Rc<Node<T>>>,
            next: Option<Rc<Node<T>>>,
            node: &Rc<Node<T>>,
        ) {
            if let Some(prev) = &prev {
                prev.next_sibling.set(Rc::downgrade(node));
            } else {
                parent.first_child.set(Rc::downgrade(node));
            }

            if let Some(next) = &next {
                next.prev_sibling.set(Rc::downgrade(node));
            } else {
                parent.last_child.set(Rc::downgrade(node));
            }

            node.parent.set(Rc::downgrade(&parent));
            node.prev_sibling
                .set(prev.as_ref().map(Rc::downgrade).unwrap_or_default());
            node.next_sibling
                .set(next.as_ref().map(Rc::downgrade).unwrap_or_default());
        }
    }

    pub fn children(&self) -> Iter<T> {
        Iter {
            node: self.first_child().upgrade(),
        }
    }

    pub fn parents(&self) -> Parents<T> {
        Parents {
            node: self.parent().upgrade(),
        }
    }
}

impl<T> Drop for Node<T> {
    fn drop(&mut self) {
        let mut node = self.first_child.get();
        while node.strong_count() > 0 {
            let n = unsafe { Rc::from_raw(node.as_ptr()) };
            n.parent.take();
            n.prev_sibling.take();
            node = n.next_sibling.take();
        }
    }
}

pub struct Iter<T> {
    node: Option<Rc<Node<T>>>,
}

impl<T> Deref for Node<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.value
    }
}

impl<T> DerefMut for Node<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.value
    }
}

impl<T> Iterator for Iter<T> {
    type Item = Rc<Node<T>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.node.take().map(|node| {
            self.node = node.next_sibling().upgrade();
            node
        })
    }
}

pub struct Parents<T> {
    node: Option<Rc<Node<T>>>,
}

impl<T> Iterator for Parents<T> {
    type Item = Rc<Node<T>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.node.take().map(|node| {
            self.node = node.parent().upgrade();
            node
        })
    }
}
