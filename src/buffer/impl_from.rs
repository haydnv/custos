use crate::{Alloc, Buffer, GraphReturn};

impl<'a, T, D, const N: usize> From<(&'a D, [T; N])> for Buffer<'a, T, D>
where
    T: Clone,
    D: Alloc<T> + GraphReturn,
    // TODO: remove later for stack impl
    D::Ptr<T, 0>: Default,
{
    fn from(device_slice: (&'a D, [T; N])) -> Self {
        let len = device_slice.1.len();
        Buffer {
            // TODO: with_array()
            ptr: device_slice.0.with_slice(&device_slice.1),
            len,
            device: Some(device_slice.0),
            node: device_slice.0.graph().add_leaf(len),
            ..Default::default()
        }
    }
}

impl<'a, T, D, const N: usize> From<(&'a D, &[T; N])> for Buffer<'a, T, D>
where
    T: Clone,
    D: Alloc<T> + GraphReturn,
    D::Ptr<T, 0>: Default,
{
    fn from(device_slice: (&'a D, &[T; N])) -> Self {
        let len = device_slice.1.len();
        Buffer {
            // TODO: with_array()
            ptr: device_slice.0.with_slice(device_slice.1),
            len,
            device: Some(device_slice.0),
            node: device_slice.0.graph().add_leaf(len),
            ..Default::default()
        }
    }
}

impl<'a, T, D> From<(&'a D, &[T])> for Buffer<'a, T, D>
where
    T: Clone,
    D: Alloc<T> + GraphReturn,
    D::Ptr<T, 0>: Default,
{
    fn from(device_slice: (&'a D, &[T])) -> Self {
        let len = device_slice.1.len();
        Buffer {
            ptr: device_slice.0.with_slice(device_slice.1),
            len,
            device: Some(device_slice.0),
            node: device_slice.0.graph().add_leaf(len),
            ..Default::default()
        }
    }
}

impl<'a, T, D> From<(&'a D, Vec<T>)> for Buffer<'a, T, D>
where
    T: Clone,
    D: Alloc<T> + GraphReturn,
    D::Ptr<T, 0>: Default,
{
    fn from(device_vec: (&'a D, Vec<T>)) -> Self {
        let len = device_vec.1.len();
        Buffer {
            ptr: device_vec.0.alloc_with_vec(device_vec.1),
            len,
            device: Some(device_vec.0),
            node: device_vec.0.graph().add_leaf(len),
            ..Default::default()
        }
    }
}

impl<'a, T, D> From<(&'a D, &Vec<T>)> for Buffer<'a, T, D>
where
    T: Clone,
    D: Alloc<T> + GraphReturn,
    D::Ptr<T, 0>: Default,
{
    fn from(device_slice: (&'a D, &Vec<T>)) -> Self {
        let len = device_slice.1.len();
        Buffer {
            ptr: device_slice.0.with_slice(device_slice.1),
            len,
            device: Some(device_slice.0),
            node: device_slice.0.graph().add_leaf(len),
            ..Default::default()
        }
    }
}
