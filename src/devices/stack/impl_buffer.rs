use crate::{Buffer, BufFlag, Node};
use super::stack_device::{Stack, StackArray};

impl<'a, T, const N: usize> From<[T; N]> for Buffer<'a, T, Stack, N> {
    fn from(array: [T; N]) -> Self {
        Buffer {
            ptr: StackArray::new(array),
            len: N,
            device: Some(&Stack),
            flag: BufFlag::None,
            node: Node::default(),
        }
    }
}