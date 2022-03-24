use std::fmt::Debug;

use crate::{AsDev, BaseDevice, BaseOps, Buffer, Device, Gemm, libs::{cpu::{CPUCache, ops::element_wise_op_mut}, opencl::GenericOCL}, matrix::Matrix, VecRead, number::Number, Dealloc, Threaded};

use super::TBlas;

#[derive(Debug, Clone, Copy)]
pub struct CPU;

impl CPU {
    pub fn drop<T>(buf: Buffer<T>) {
        unsafe {    
            drop(Box::from_raw(buf.ptr));
        }
    }
    pub fn mt<T: Default+Copy>(self) -> (Self, Threaded<CPU>) {
        (self, Threaded::new(self))
    }
}

impl <T: TBlas+Default+Copy>Gemm<T> for CPU {
    fn gemm(&self, lhs: Matrix<T>, rhs: Matrix<T>) -> Matrix<T> {
        let m = lhs.dims().0;
        let k = lhs.dims().1;
        let n = rhs.dims().1;

        let mut c = CPUCache::get((m, n));
        T::gemm(m, n, k, lhs.as_cpu_slice(), rhs.as_cpu_slice(), c.as_cpu_slice_mut());
        c
    }
}

impl <T: GenericOCL+TBlas>BaseDevice<T> for CPU {}

pub fn ew_op<T: Copy+Default, F: Fn(T, T) -> T>(lhs: Matrix<T>, rhs: Matrix<T>, f: F) -> Matrix<T> {
    let mut out = CPUCache::get::<T>(lhs.dims());
    element_wise_op_mut(lhs.as_cpu_slice(), rhs.as_cpu_slice(), out.as_cpu_slice_mut(), f);
    out
}

impl <T: Number>BaseOps<T> for CPU {
    fn add(&self, lhs: Matrix<T>, rhs: Matrix<T>) -> Matrix<T> {
        ew_op(lhs, rhs, | x, y| x+y)
    }

    fn sub(&self, lhs: Matrix<T>, rhs: Matrix<T>) -> Matrix<T> {
        ew_op(lhs, rhs, | x, y| x-y)
    }

    fn mul(&self, lhs: Matrix<T>, rhs: Matrix<T>) -> Matrix<T> {
        ew_op(lhs, rhs, | x, y| x*y)
    }
}


impl AsDev for CPU {
    fn as_dev(&self) -> crate::Dev {
        crate::Dev::new(None)
    }
}

impl <T: Default+Copy>Device<T> for CPU {
    fn alloc(&self, len: usize) -> *mut T {
        Box::into_raw(vec![T::default(); len].into_boxed_slice()) as *mut T
    }

    fn with_data(&self, data: &[T]) -> *mut T {
        Box::into_raw(data.to_vec().into_boxed_slice()) as *mut T
    }
}

/* 
impl Device for CPU {
    fn alloc<T: Default+Copy>(&self, len: usize) -> *mut T {
        Box::into_raw(vec![T::default(); len].into_boxed_slice()) as *mut T
    }

    fn from_data<T: Clone>(&self, data: &[T]) -> *mut T {
        Box::into_raw(data.to_vec().into_boxed_slice()) as *mut T
    }
}
*/

impl <T: Copy+Default>VecRead<T> for CPU {
    fn read(&self, buf: crate::Buffer<T>) -> Vec<T> {
        unsafe {
            std::slice::from_raw_parts(buf.ptr, buf.len).to_vec()
        }
    }
}

impl Dealloc for CPU {
    fn dealloc_cache(&self) {
        /* 
        let mut cache = CPU_CACHE.lock().unwrap();
        
        let contents = cache.nodes.clone();

        contents.into_iter()
            .filter(|entry| entry.0.thread_id == std::thread::current().id())
            .for_each(|entry| {
                let ptr = (entry.1).0;
                unsafe { Box::from_raw(ptr.0) };
                cache.nodes.remove(&entry.0);
        });
        */
    }
}