//! A minimal OpenCL, CUDA and host CPU array manipulation engine / framework written in Rust.
//! This crate provides the tools for executing custom array operations with the CPU, as well as with CUDA and OpenCL devices.<br>
//! This guide demonstrates how operations can be implemented for the compute devices: [implement_operations.md](implement_operations.md)<br>
//! or to see it at a larger scale, look here: [custos-math]
//!
//! ## [Examples]
//!
//! [examples]: https://github.com/elftausend/custos/tree/main/examples
//!
//! Using the host CPU as the compute device:
//!
//! [cpu_readme.rs]
//!
//! [cpu_readme.rs]: https://github.com/elftausend/custos/blob/main/examples/cpu_readme.rs
//!
//! ```rust
//! use custos::{CPU, ClearBuf, VecRead, Buffer};
//!
//! let device = CPU::new();
//! let mut a = Buffer::from(( &device, [1, 2, 3, 4, 5, 6]));
//!     
//! // specify device for operation
//! device.clear(&mut a);
//! assert_eq!(device.read(&a), [0; 6]);
//!
//! let device = CPU::new();
//!
//! let mut a = Buffer::from(( &device, [1, 2, 3, 4, 5, 6]));
//! a.clear();
//!
//! assert_eq!(a.read(), vec![0; 6]);
//! ```
use std::ffi::c_void;

//pub use libs::*;
pub use buffer::*;
pub use count::*;
pub use devices::*;
pub use graph::*;
pub use error::*;

pub use devices::cpu::CPU;
#[cfg(feature = "cuda")]
pub use devices::cuda::CUDA;
#[cfg(feature = "opencl")]
pub use devices::opencl::{OpenCL, InternCLDevice};

pub mod devices;

mod buffer;
mod count;
mod graph;
mod error;

pub mod number;


thread_local! {
    pub static GLOBAL_CPU: CPU = CPU::new();
}

#[derive(Debug, Clone, Copy)]
pub struct Deviceless;
pub trait DevicelessAble: Alloc {}

/// This trait is for allocating memory on the implemented device.
///
/// # Example
/// ```
/// use custos::{CPU, Alloc, Buffer, VecRead, BufFlag, GraphReturn};
///
/// let device = CPU::new();
/// let ptrs: (*mut f32, *mut std::ffi::c_void, u64) = device.alloc(12);
///
/// let buf = Buffer {
///     ptr: ptrs,
///     len: 12,
///     device: Some(&device),
///     flag: BufFlag::None,
///     node: device.graph().add_leaf(12),
/// };
/// assert_eq!(vec![0.; 12], device.read(&buf));
/// ```
pub trait Alloc {
    /// Allocate memory on the implemented device.
    /// # Example
    /// ```
    /// use custos::{CPU, Alloc, Buffer, VecRead, BufFlag, GraphReturn};
    ///
    /// let device = CPU::new();
    /// let ptrs: (*mut f32, *mut std::ffi::c_void, u64) = device.alloc(12);
    ///
    /// let buf = Buffer {
    ///     ptr: ptrs,
    ///     len: 12,
    ///     device: Some(&device),
    ///     flag: BufFlag::None,
    ///     node: device.graph().add_leaf(12),
    /// };
    /// assert_eq!(vec![0.; 12], device.read(&buf));
    /// ```
    fn alloc<T>(&self, len: usize) -> (*mut T, *mut c_void, u64);

    /// Allocate new memory with data
    /// # Example
    /// ```
    /// use custos::{CPU, Alloc, Buffer, VecRead, BufFlag, GraphReturn};
    ///
    /// let device = CPU::new();
    /// let ptrs: (*mut u8, *mut std::ffi::c_void, u64) = device.with_data(&[1, 5, 4, 3, 6, 9, 0, 4]);
    ///
    /// let buf = Buffer {
    ///     ptr: ptrs,
    ///     len: 8,
    ///     device: Some(&device),
    ///     flag: BufFlag::None,
    ///     node: device.graph().add_leaf(8),
    /// };
    /// assert_eq!(vec![1, 5, 4, 3, 6, 9, 0, 4], device.read(&buf));
    /// ```
    fn with_data<T>(&self, data: &[T]) -> (*mut T, *mut c_void, u64)
    where
        T: Clone;

    /// If the vector `vec` was allocated previously, this function can be used in order to reduce the amount of allocations, which may be faster than using a slice of `vec`.
    fn alloc_with_vec<T>(&self, vec: Vec<T>) -> (*mut T, *mut c_void, u64)
    where
        T: Clone,
    {
        self.with_data(&vec)
    }
}

/// Trait for implementing the clear() operation for the compute devices.
pub trait ClearBuf<T> {
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
    fn clear(&self, buf: &mut Buffer<T, Self>) where Self: Sized;
}

/// Trait for reading buffers.
pub trait VecRead<T>: Sized {
    /// Read the data of a buffer into a vector
    /// # Example
    /// ```
    /// use custos::{CPU, Buffer, VecRead};
    ///
    /// let device = CPU::new();
    /// let a = Buffer::from((&device, [1., 2., 3., 3., 2., 1.,]));
    /// let read = device.read(&a);
    /// assert_eq!(vec![1., 2., 3., 3., 2., 1.,], read);
    /// ```
    fn read(&self, buf: &Buffer<T, Self>) -> Vec<T>;
}

/// Trait for writing data to buffers.
pub trait WriteBuf<T>: Sized {
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
    fn write(&self, buf: &mut Buffer<T, Self>, data: &[T]);
    /// Writes data from <Device> Buffer to other <Device> Buffer.
    // TODO: implement, change name of fn? -> set_.. ?
    fn write_buf(&self, _dst: &mut Buffer<T, Self>, _src: &Buffer<T, Self>) {
        unimplemented!()
    }
}

/// This trait is used to clone a buffer based on a specific device type.
pub trait CloneBuf<'a, T>: Sized {
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
    fn clone_buf(&'a self, buf: &Buffer<'a, T, Self>) -> Buffer<'a, T, Self>;
}

/// This trait is used to retrieve a cached buffer from a specific device type.
pub trait CacheBuf<'a, T> where Self: Sized {
    #[cfg_attr(feature = "realloc", doc = "```ignore")]
    /// Adds a buffer to the cache. Following calls will return this buffer, if the corresponding internal count matches with the id used in the cache.
    /// # Example
    /// ```
    /// use custos::{CPU, VecRead, set_count, get_count, CacheBuf};
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
    /// set_count(0);
    /// let buf = CacheBuf::<f32>::cached(&device, 10);
    /// assert_eq!(device.read(&buf), vec![1.5; 10]);
    /// ```
    fn cached(&'a self, len: usize) -> Buffer<'a, T, Self>;
}