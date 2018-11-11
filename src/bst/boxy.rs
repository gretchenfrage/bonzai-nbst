
use super::Bst;

use std::cmp::Ordering;
use std::iter::{IntoIterator, Iterator};
use std::fmt::Debug;

#[derive(Debug)]
pub struct BoxBst<T: Ord + Debug> {
    root: Option<BoxBstNode<T>>
}
impl<T: Ord + Debug> Bst<T> for BoxBst<T> {
    fn new() -> Self {
        BoxBst {
            root: None
        }
    }

    fn insert(&mut self, elem: T) -> bool {
        match self.root {
            Some(ref mut root) => root.insert(elem),
            None => {
                self.root = Some(BoxBstNode::new(elem));
                true
            }
        }
    }

    fn remove(&mut self, elem: &T) -> bool {
        if let Some(root) = self.root.take() {
            let (new_root, removed) = root.remove(elem);
            if let Some(new_root) = new_root {
                self.root = Some(new_root);
            }
            removed
        } else {
            false
        }
    }

    fn contains(&self, elem: &T) -> bool {
        match self.root {
            Some(ref root) => root.contains(elem),
            None => false
        }
    }
}
impl<'s, T: Ord + Debug> IntoIterator for &'s BoxBst<T> {
    type Item = &'s T;
    type IntoIter = Iter<'s, T>;

    fn into_iter(self) -> Iter<'s, T> {
        match self.root {
            Some(ref root) => Iter::new(root),
            None => Iter {
                frames: Vec::new(),
            }
        }
    }
}

#[derive(Debug)]
struct BoxBstNode<T: Ord + Debug> {
    elem: T,
    children: [Option<Box<BoxBstNode<T>>>; 2],
}
impl<T: Ord + Debug> BoxBstNode<T> {
    fn new(elem: T) -> Self {
        BoxBstNode {
            elem,
            children: [None, None],
        }
    }

    fn insert(&mut self, elem: T) -> bool {
        let recurse_into: usize = match elem.cmp(&self.elem) {
            Ordering::Equal => {
                return false;
            },
            Ordering::Greater => 1,
            Ordering::Less => 0,
        };
        match &mut self.children[recurse_into] {
            &mut Some(ref mut child) => child.insert(elem),
            None => {
                self.children[recurse_into] = Some(Box::new(BoxBstNode::new(elem)));
                true
            }
        }
    }

    fn remove(mut self, elem: &T) -> (Option<Self>, bool) {
        let recurse_into: Option<usize> = match elem.cmp(&self.elem) {
            Ordering::Equal => None,
            Ordering::Greater => Some(1),
            Ordering::Less => Some(0),
        };
        match recurse_into {
            Some(branch) => {
                // the node belongs in a child
                // detach the child
                match self.children[branch].take() {
                    Some(child) => {
                        // move the detached child through a recursion of this function
                        let (new_child, removed) = child.remove(elem);
                        if let Some(new_child) = new_child {
                            // if it produced a replacement child, reattach it
                            self.children[branch] = Some(Box::new(new_child));
                        }
                        // remain self
                        (Some(self), removed)
                    },
                    None => {
                        // there is no match
                        (Some(self), false)
                    }
                }
            },
            None => {
                // this node is the element being removed
                match (
                    self.children[0].take(),
                    self.children[1].take(),
                ) {
                    (None, None) => {
                        // no children, simply remove self
                        (None, true)
                    },
                    (Some(left), None) => {
                        // only left child is present, become left child
                        (Some(*left), true)
                    },
                    (None, Some(right)) => {
                        // only right child is present, become right child
                        (Some(*right), true)
                    },
                    (Some(left), Some(right)) => {
                        // both children are present
                        // remove the leftmost element of the right child
                        let (new_right, new_self_elem) = right.detach_leftmost();
                        // become that element, and reattach both child trees
                        let mut new_self_node = Self::new(new_self_elem);
                        new_self_node.children[0] = Some(left);
                        if let Some(new_right) = new_right {
                            new_self_node.children[1] = Some(Box::new(new_right));
                        }
                        (Some(new_self_node), true)
                    }
                }
            }
        }
    }

    fn detach_leftmost(mut self) -> (Option<Self>, T) {
        match self.children[0].take() {
            Some(left_child) => {
                // try to recurse to the left child
                let (new_child, elem) = left_child.detach_leftmost();
                if let Some(new_child) = new_child {
                    self.children[0] = Some(Box::new(new_child));
                }
                (Some(self), elem)
            },
            None => {
                // if no left child exists, detach this elem, and become right child, if present
                let right_child = self.children[1].take().map(|boxed| *boxed);
                (right_child, self.elem)
            }
        }
    }

    fn contains(&self, elem: &T) -> bool {
        let recurse_into: usize = match elem.cmp(&self.elem) {
            Ordering::Equal => {
                return true;
            },
            Ordering::Greater => 1,
            Ordering::Less => 0,
        };
        match self.children[recurse_into] {
            Some(ref child) => child.contains(elem),
            None => false,
        }
    }
}

pub struct Iter<'t, T: Ord + Debug> {
    frames: Vec<IterFrame<&'t BoxBstNode<T>>>,
}
#[derive(Copy, Clone)]
struct IterFrame<E> {
    elem: E,
    branch: Option<usize>
}
impl<'t, T: Ord + Debug> Iter<'t, T> {
    fn new(root: &'t BoxBstNode<T>) -> Self {
        // initially seek the leftmost node
        let mut iter = Iter {
            frames: vec![IterFrame {
                elem: root,
                branch: None,
            }],
        };
        iter.seek_leftmost();
        iter
    }

    fn seek_leftmost(&mut self) {
        while match &self.frames.last().unwrap().elem.children[0] {
            Some(ref left_child) => {
                self.frames.push(IterFrame {
                    elem: &*left_child,
                    branch: Some(0),
                });
                true
            },
            None => match &self.frames.last().unwrap().elem.children[1] {
                Some(ref right_child) => {
                    self.frames.push(IterFrame {
                        elem: &*right_child,
                        branch: Some(1),
                    });
                    true
                },
                None => false
            }
        } {}
    }
}
impl<'t, T: Ord + Debug> Iterator for Iter<'t, T> {
    type Item = &'t T;

    fn next(&mut self) -> Option<&'t T> {
        match self.frames.last().cloned() {
            Some(curr_frame) => {
                // move up, and if we moved up from the left child, seek the leftmost node
                // of the right subtree
                match curr_frame.branch {
                    Some(this_branch_index) => {
                        self.frames.pop().unwrap();
                        if this_branch_index == 0 {
                            match self.frames.last().unwrap().elem.children[1].as_ref() {
                                Some(right_child) => {
                                    self.frames.push(IterFrame {
                                        elem: &*right_child,
                                        branch: Some(1),
                                    });
                                    // only seek the leftmost child if we actually have a right child
                                    self.seek_leftmost();
                                },
                                None => ()
                            };
                        }
                    },
                    None => {
                        // however, if we've hit the top, that means that we're done iterating
                        self.frames.pop().unwrap();
                        debug_assert_eq!(self.frames.len(), 0);
                    }
                };

                Some(&curr_frame.elem.elem)
            },
            None => None
        }
    }
}