use crate::{shape::Shape, Buffer, Device, Eval, Resolve, Alloc, MayTapeReturn};

/// Trait for implementing the clear() operation for the compute devices.
pub trait ClearBuf<T, D: Device = Self, S: Shape = ()>: Device {
    /// Sets all elements of the matrix to zero.
    /// # Example
    /// ```
    /// use custos::{CPU, ClearBuf, Buffer};
    ///
    /// let device = CPU::new();
    /// let mut a = Buffer::from((&device, [2, 4, 6, 8, 10, 12]));
    /// assert_eq!(a.read(), vec![2, 4, 6, 8, 10, 12]);
    ///
    /// device.clear(&mut a);
    /// assert_eq!(a.read(), vec![0; 6]);
    /// ```
    fn clear(&self, buf: &mut Buffer<T, D, S>);
}

/// Trait for reading buffers.
pub trait Read<T, D: Device = Self, S: Shape = ()>: Device {
    type Read<'a>
    where
        T: 'a,
        D: 'a,
        S: 'a;

    /// Read the data of the `Buffer` as type `Read`.
    /// # Example
    /// ```
    /// use custos::{CPU, Buffer, Read};
    ///
    /// let device = CPU::new();
    /// let a = Buffer::from((&device, [1., 2., 3., 3., 2., 1.,]));
    /// let read = device.read(&a);
    /// assert_eq!(&[1., 2., 3., 3., 2., 1.,], read);
    /// ```
    fn read<'a>(&self, buf: &'a Buffer<T, D, S>) -> Self::Read<'a>;

    /// Read the data of a buffer into a vector
    /// # Example
    /// ```
    /// use custos::{CPU, Buffer, Read};
    ///
    /// let device = CPU::new();
    /// let a = Buffer::from((&device, [1., 2., 3., 3., 2., 1.,]));
    /// let read = device.read_to_vec(&a);
    /// assert_eq!(vec![1., 2., 3., 3., 2., 1.,], read);
    /// ```
    #[cfg(not(feature = "no-std"))]
    fn read_to_vec(&self, buf: &Buffer<T, D, S>) -> Vec<T>
    where
        T: Default + Clone;
}

/// Trait for writing data to buffers.
pub trait WriteBuf<T, D: Device = Self, S: Shape = ()>: Sized + Device {
    /// Write data to the buffer.
    /// # Example
    /// ```
    /// use custos::{CPU, Buffer, WriteBuf};
    ///
    /// let device = CPU::new();
    /// let mut buf = Buffer::new(&device, 4);
    /// device.write(&mut buf, &[9, 3, 2, -4]);
    /// assert_eq!(buf.as_slice(), &[9, 3, 2, -4])
    ///
    /// ```
    fn write(&self, buf: &mut Buffer<T, D, S>, data: &[T]);
    /// Writes data from <Device> Buffer to other <Device> Buffer.
    // TODO: implement, change name of fn? -> set_.. ?
    fn write_buf(&self, _dst: &mut Buffer<T, Self, S>, _src: &Buffer<T, Self, S>) {
        unimplemented!()
    }
}

/// This trait is used to clone a buffer based on a specific device type.
pub trait CloneBuf<'a, T, S: Shape = ()>: Sized + Device {
    /// Creates a deep copy of the specified buffer.
    /// # Example
    ///
    /// ```
    /// use custos::{CPU, Buffer, CloneBuf};
    ///
    /// let device = CPU::new();
    /// let buf = Buffer::from((&device, [1., 2., 6., 2., 4.,]));
    ///
    /// let cloned = device.clone_buf(&buf);
    /// assert_eq!(buf.read(), cloned.read());
    /// ```
    fn clone_buf(&'a self, buf: &Buffer<'a, T, Self, S>) -> Buffer<'a, T, Self, S>;
}

/// This trait is used to retrieve a cached buffer from a specific device type.
pub trait CacheBuf<'a, T, S: Shape = ()>: Sized + Device {
    /// Adds a buffer to the cache. Following calls will return this buffer, if the corresponding internal count matches with the id used in the cache.
    /// # Example
    #[cfg_attr(feature = "realloc", doc = "```ignore")]
    #[cfg_attr(not(feature = "realloc"), doc = "```")]
    /// use custos::{CPU, Read, set_count, get_count, CacheBuf};
    ///
    /// let device = CPU::new();
    /// assert_eq!(0, get_count());
    ///
    /// let mut buf = CacheBuf::<f32>::cached(&device, 10);
    /// assert_eq!(1, get_count());
    ///
    /// for value in buf.as_mut_slice() {
    ///     *value = 1.5;
    /// }
    ///    
    /// unsafe { set_count(0) };
    /// let buf = CacheBuf::<f32>::cached(&device, 10);
    /// assert_eq!(device.read(&buf), vec![1.5; 10]);
    /// ```
    fn cached(&'a self, len: usize) -> Buffer<'a, T, Self, S>;
}

pub trait ApplyFunction<T, S: Shape = (), D: Device = Self>: Device {
    fn apply_fn<F>(&self, buf: &Buffer<T, D, S>, f: impl Fn(Resolve<T>) -> F) -> Buffer<T, Self, S>
    where
        F: Eval<T> + ToString;
}

pub trait UnaryGrad<T, S: Shape = (), D: Device = Self>: Device {
    fn add_unary_grad<F>(
        &self,
        lhs: &Buffer<T, D, S>,
        lhs_grad: &mut Buffer<T, D, S>,
        out: &Buffer<T, D, S>,
        lhs_grad_fn: impl Fn(Resolve<T>) -> F,
    ) where
        F: Eval<T> + ToString;
}

pub trait UnaryElementWise<T, D: Device, S: Shape>: Device {
    fn unary_ew<FO, GO>(
        &self,
        buf: &Buffer<T, D, S>,
        forward_fn: impl Fn(Resolve<T>) -> FO,
        grad_fn: fn(Resolve<T>) -> GO,
    ) -> Buffer<T, Self, S>
    where
        FO: Eval<T> + ToString,
        GO: Eval<T> + ToString + 'static;
}

impl<T, D, S> UnaryElementWise<T, D, S> for D
where
    T: 'static,
    D: ApplyFunction<T, S, D> + UnaryGrad<T, S, D> + MayTapeReturn,
    D: for<'a> Alloc<'a, T, S>,
    S: Shape,
{
    fn unary_ew<FO, GO>(
        &self,
        buf: &Buffer<T, D, S>,
        forward_fn: impl Fn(Resolve<T>) -> FO,
        _grad_fn: fn(Resolve<T>) -> GO,
    ) -> Buffer<T, Self, S>
    where
        FO: Eval<T> + ToString,
        GO: Eval<T> + ToString + 'static,
    {
        let out = self.apply_fn(buf, forward_fn);

        #[cfg(feature = "autograd")]
        {
            let ids = (buf.id(), out.id());
            self.tape_mut().add_grad_fn(move |grads, device| {
                let (lhs, mut lhs_grad, out_grad) = grads.get_double::<T, S>(device, ids);
                device.add_unary_grad(&lhs, &mut lhs_grad, &out_grad, _grad_fn);
            });
        }

        out
    }
}

#[cfg(test)]
mod tests {

    #[cfg(feature="stack")]
    #[cfg(not(feature="autograd"))]
    #[test]
    fn test_unary_ew_stack_no_autograd() {
        use crate::{Buffer, Dim1, UnaryElementWise, Combiner};

        let device = crate::Stack;
        let buf = Buffer::<_, _, Dim1<5>>::from((&device, [1, 2, 4, 5, 3]));
        
        let out = device.unary_ew(&buf, |x| x.mul(3), |x| x);
        
        assert_eq!(out.read(), [3, 6, 12, 15, 9]);
    }
}