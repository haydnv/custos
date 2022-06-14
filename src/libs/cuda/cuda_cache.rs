use std::{collections::HashMap, cell::RefCell};
use crate::{Node, InternCudaDevice, Buffer};

thread_local! {
    pub static CUDA_CACHE: RefCell<CudaCache> = RefCell::new(CudaCache { 
        nodes: HashMap::new(), 
    })
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct CudaPtr(pub u64);

unsafe impl Send for CudaPtr {}
unsafe impl Sync for CudaPtr {}

type RawInfo = (CudaPtr, usize);

pub struct CudaCache {
    pub nodes: HashMap<Node, RawInfo>,
}

impl CudaCache {
    pub fn add_node<T:>(&mut self, device: InternCudaDevice, node: Node) -> Buffer<T> {
        let out = Buffer::new(&device, node.len);
        self.nodes.insert(node, ( CudaPtr(out.ptr.2), out.len ));
        out
    }

    #[cfg(not(feature="safe"))]
    pub fn get<T: >(device: InternCudaDevice, len: usize) -> Buffer<T> {
        use std::ptr::null_mut;
        
        assert!(!device.cuda.borrow().ptrs.is_empty(), "no Cuda allocations");
        let node = Node::new(len);

        CUDA_CACHE.with(|cache| {
            let mut cache = cache.borrow_mut();
            let buf_info_option = cache.nodes.get(&node);
    
            match buf_info_option {
                Some(buf_info) => {
            
                    Buffer {
                        ptr: (null_mut(), null_mut(), buf_info.0.0),
                        len: buf_info.1
                    }
                }
                None => cache.add_node(device, node)
            }
        })
    }

    #[cfg(feature="safe")]
    pub fn get<T: GenericOCL>(device: InternCLDevice, len: usize) -> Buffer<T> {
        Buffer::new(&device, len)
    }
}
