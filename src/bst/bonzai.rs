
use super::Bst;

use std::cmp::Ordering;
use std::mem;
use std::iter::{IntoIterator, Iterator};
use std::fmt::Debug;

use bonzai::*;

#[derive(Debug)]
pub struct BonzaiBst<T: Ord + Debug> {
    tree: Tree<T, [ChildId; 2]>,
}
impl<T: Ord + Debug> Bst<T> for BonzaiBst<T> {
    fn new() -> Self {
        BonzaiBst {
            tree: Tree::new(),
        }
    }

    fn insert(&mut self, elem: T) -> bool {
        let mut op = self.tree.operation();
        match op.write_root() {
            Some(root) => insert_node(root, elem),
            None => {
                op.put_root_elem(elem);
                true
            },
        }
    }

    fn remove(&mut self, elem: &T) -> bool {
        let op = self.tree.operation();
        let removed = match op.take_root() {
            Some(root) => {
                let (new_root, removed) = remove_node(root, elem);
                if let Some(new_root) = new_root {
                    op.try_put_root_tree(new_root).unwrap();
                }
                removed
            },
            None => false
        };
        removed
    }

    fn contains(&self, elem: &T) -> bool {
        match self.tree.read_root() {
            Some(root) => node_contains(root, elem),
            None => false,
        }
    }
}
impl<'s, T: Ord + Debug> IntoIterator for &'s BonzaiBst<T> {
    type Item = &'s T;
    type IntoIter = Iter<'s, T>;

    fn into_iter(self) -> Iter<'s, T> {
        match self.tree.traverse_read_root() {
            Some(trav) => Iter::new(trav),
            None => Iter {
                traverser: None
            }
        }
    }
}

fn insert_node<T: Ord>(node: NodeWriteGuard<T, [ChildId; 2]>, elem: T) -> bool {
    let (node_elem, mut children) = node.into_split();
    let recurse_into: usize = match elem.cmp(node_elem) {
        Ordering::Equal => {
            return false;
        },
        Ordering::Greater => 1,
        Ordering::Less => 0,
    };
    match children.borrow_child_write(recurse_into).unwrap() {
        Some(child) => insert_node(child, elem),
        None => {
            children.put_child_elem(recurse_into, elem).unwrap();
            true
        }
    }
}

fn remove_node<'o, 't: 'o, T: Ord>(mut node: NodeOwnedGuard<'o, 't, T, [ChildId; 2]>, elem: &T)
    -> (Option<NodeOwnedGuard<'o, 't, T, [ChildId; 2]>>, bool) {
    let recurse_into: Option<usize> = match elem.cmp(&*node.elem()) {
        Ordering::Equal => None,
        Ordering::Greater => Some(1),
        Ordering::Less => Some(0),
    };
    let mut children = node.children();
    match recurse_into {
        Some(branch) => {
            // the node belongs in a child
            // detach the child
            match children.take_child(branch).unwrap() {
                Some(child) => {
                    // move the detached child through a recursion of this function
                    let (new_child, removed) = remove_node(child, elem);
                    if let Some(new_child) = new_child {
                        // if it produced a replacement child, reattach it
                        children.put_child_tree(branch, new_child).unwrap();
                    }
                    // remain self
                    mem::drop(children);
                    (Some(node), removed)
                },
                None => {
                    // there is no match
                    mem::drop(children);
                    (Some(node), false)
                }
            }
        },
        None => {
            // this node is the element being removed
            match (
                children.take_child(0).unwrap(),
                children.take_child(1).unwrap(),
            ) {
                (None, None) => {
                    // no children, simply remove self
                    (None, true)
                },
                (Some(left), None) => {
                    // no children, simply remove self
                    (Some(left), true)
                },
                (None, Some(right)) => {
                    // only right child is present, become right child
                    (Some(right), true)
                },
                (Some(left), Some(right)) => {
                    // both children are present
                    // remove the leftmost element of the right child
                    let (new_right, new_self_elem) = detach_leftmost(right);
                    // become that element, and reattach both child trees
                    let mut new_self_tree = node.op.new_detached(new_self_elem);
                    {
                        let mut new_self_children = new_self_tree.children();
                        new_self_children.put_child_tree(0, left).unwrap();
                        if let Some(new_right) = new_right {
                            new_self_children.put_child_tree(1, new_right).unwrap();
                        }
                    }
                    (Some(new_self_tree), true)
                }
            }
        }
    }
}

fn detach_leftmost<'o, 't: 'o, T: Ord>(mut node: NodeOwnedGuard<'o, 't, T, [ChildId; 2]>)
    -> (Option<NodeOwnedGuard<'o, 't, T, [ChildId; 2]>>, T) {

    let mut children = node.children();
    match children.take_child(0).unwrap() {
        Some(left_child) => {
            // try to recurse to the left child
            let (new_child, elem) = detach_leftmost(left_child);
            if let Some(new_child) = new_child {
                children.put_child_tree(0, new_child).unwrap();
            }
            mem::drop(children);
            return (Some(node), elem);
        },
        None => {
            // if no left child exists, detach this elem, and become right child, if present
            let right_child = children.take_child(1).unwrap();
            mem::drop(children);
            (right_child, node.into_elem())
        }
    }
}

fn node_contains<T: Ord>(node: NodeReadGuard<T, [ChildId; 2]>, elem: &T) -> bool {
    let recurse_into: usize = match elem.cmp(&*node) {
        Ordering::Equal => {
            return true;
        },
        Ordering::Greater => 1,
        Ordering::Less => 0,
    };
    match node.child(recurse_into).unwrap() {
        Some(child) => node_contains(child, elem),
        None => false
    }
}

pub struct Iter<'t, T: Ord + Debug> {
    traverser: Option<TreeReadTraverser<'t, T, [ChildId; 2]>>
}
impl<'t, T: Ord + Debug> Iter<'t, T> {
    fn new(mut trav: TreeReadTraverser<'t, T, [ChildId; 2]>) -> Self {
        // initially seek the leftmost node
        Self::seek_leftmost(&mut trav);
        Iter {
            traverser: Some(trav)
        }
    }

    fn seek_leftmost(trav: &TreeReadTraverser<'t, T, [ChildId; 2]>) {
        while match trav.seek_child(0).unwrap() {
            Ok(_) => true,
            Err(_) => match trav.seek_child(1).unwrap() {
                Ok(_) => true,
                Err(_) => false
            }
        } {}
    }
}
impl<'t, T: Ord + Debug> Iterator for Iter<'t, T> {
    type Item = &'t T;

    fn next(&mut self) -> Option<&'t T> {
        match self.traverser {
            Some(ref mut trav) => {
                let curr = trav.elem();

                // move up, and if we moved up from the left child, seek the leftmost node
                // of the right subtree
                match trav.this_branch_index() {
                    Ok(this_branch_index) => {
                        trav.seek_parent().unwrap();
                        if this_branch_index == 0 {
                            if trav.seek_child(1).unwrap().is_ok() {
                                // only seek the leftmost child if we actually have a right child
                                Self::seek_leftmost(trav);
                            }
                        }
                    },
                    Err(_) => {
                        // however, if we've hit the top, that means that we're done iterating
                        mem::drop(trav);
                        self.traverser = None;
                    }
                }

                Some(curr)
            },
            None => None
        }
    }
}