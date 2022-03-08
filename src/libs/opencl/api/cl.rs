#![allow(dead_code)]
use std::{ffi::{CString, c_void}, usize, vec};

#[cfg(feature = "nocache")]
use crate::prelude::{Tensor, OpenCL};

use super::{error::OCLError, extern_cl::*, OCLErrorKind};


#[derive(Clone, Copy)]
pub struct Platform(cl_platform_id); 

impl Platform { 
    pub fn as_ptr(self) -> *mut cl_platform_id {
        self.0 as *mut cl_platform_id
    }
}

pub fn get_platforms() -> Result<Vec<Platform>, OCLError> {
    let mut platforms: cl_uint = 0;
    let value = unsafe { clGetPlatformIDs(0, std::ptr::null_mut(), &mut platforms)};
    //ocl_error(OCLErrorKind::GetPlatformIDs, unsafe { clGetPlatformIDs(0, std::ptr::null_mut(), &mut platforms)});
    if value != 0 {
        return Err(OCLError::with_kind(OCLErrorKind::from_value(value)));
    }
    let mut vec: Vec<usize> = vec![0; platforms as usize];
    let (ptr, len, cap) = (vec.as_mut_ptr(), vec.len(), vec.capacity());

    let mut platforms_vec: Vec<Platform> = unsafe {
        core::mem::forget(vec);
        Vec::from_raw_parts(ptr as *mut Platform, len, cap)
    };

    let value = unsafe {
        clGetPlatformIDs(platforms, platforms_vec.as_mut_ptr() as *mut cl_platform_id,
    std::ptr::null_mut()
        )
    };
    if value != 0 {
        return Err(OCLError::with_kind(OCLErrorKind::from_value(value)));
    }
    Ok(platforms_vec)
}

#[derive(Clone, Copy)]
pub enum PlatformInfo {
    PlatformName = 0x0903,
}
pub fn get_platform_info(platform: Platform, param_name: PlatformInfo) -> String {
    let mut size: size_t = 0;
    unsafe {clGetPlatformInfo(platform.0, 
                            param_name as cl_platform_info, 
                       0, 
                           std::ptr::null_mut(), 
                    &mut size);};

    let mut param_value = vec![32u8; size];

    unsafe {clGetPlatformInfo(platform.0, 
                            param_name as cl_platform_info, 
                       size, 
                           param_value.as_mut_ptr() as *mut c_void, 
                    std::ptr::null_mut());};

    println!("param value: {:?}", param_value);
    String::from_utf8_lossy(&param_value).to_string()
}

pub enum DeviceType {
    DEFAULT =     (1 << 0),
    CPU =         (1 << 1),
    GPU =         (1 << 2),
    ACCELERATOR = (1 << 3),
    ALL =         0xFFFFFFFF
}

#[derive(Copy, Clone)]
pub enum DeviceInfo {
    MaxMemAllocSize = 0x1010,
    GlobalMemSize =   0x101F,
    NAME =            0x102B,
    VERSION =         0x102F,
}
#[derive(Clone, Copy, Debug, Hash)]
pub struct Device(pub cl_device_id);

impl Device {
    pub fn get_name(self) -> Result<String, OCLError> {
        Ok(get_device_info(self, DeviceInfo::NAME)?.string)
    }
    pub fn get_version(self) -> Result<String, OCLError> {
        Ok(get_device_info(self, DeviceInfo::VERSION)?.string)
    }
    pub fn get_global_mem(self) -> Result<u64, OCLError> {
        Ok(get_device_info(self, DeviceInfo::GlobalMemSize)?.size)
    }
    pub fn get_max_mem_alloc(self) -> Result<u64, OCLError> {
        Ok(get_device_info(self, DeviceInfo::MaxMemAllocSize)?.size)
    }
}


pub fn get_device_ids(platform: Platform, device_type: &u64) -> Result<Vec<Device>, OCLError> {
    let mut num_devices: cl_uint = 0;
    let value = unsafe {clGetDeviceIDs(platform.0, *device_type, 0, std::ptr::null_mut(), &mut num_devices)};
    if value != 0 {
        return Err(OCLError::with_kind(OCLErrorKind::from_value(value)));
    }

    let mut vec: Vec<usize> = vec![0; num_devices as usize];
    let (ptr, len, cap) = (vec.as_mut_ptr(), vec.len(), vec.capacity());

    let mut devices: Vec<Device> = unsafe {
        core::mem::forget(vec);
        Vec::from_raw_parts(ptr as *mut Device, len, cap)
    };

    let value = unsafe {clGetDeviceIDs(platform.0, DeviceType::GPU as u64, num_devices, devices.as_mut_ptr() as *mut cl_device_id, std::ptr::null_mut())};
    if value != 0 {
        return Err(OCLError::with_kind(OCLErrorKind::from_value(value)));
    }
    Ok(devices)
}

pub struct DeviceReturnInfo {
    pub string: String,
    pub size: u64,
}

pub fn get_device_info(device: Device, param_name: DeviceInfo) -> Result<DeviceReturnInfo, OCLError> {
    let mut size: size_t = 0;
    let value = unsafe {clGetDeviceInfo(device.0, param_name as cl_device_info, 0, std::ptr::null_mut(), &mut size)};
    if value != 0 {
        return Err(OCLError::with_kind(OCLErrorKind::from_value(value)));
    }
    let mut param_value = vec![0; size];
    let value = unsafe {clGetDeviceInfo(device.0, param_name as cl_device_info, size, param_value.as_mut_ptr() as *mut c_void, std::ptr::null_mut())};
    if value != 0 {
        return Err(OCLError::with_kind(OCLErrorKind::from_value(value)));
    }
    let string = String::from_utf8_lossy(&param_value).to_string();
    let size = param_value.iter().fold(0, |x, &i| x << 4 | i as u64);
    Ok(DeviceReturnInfo {
        string,
        size
    })
}

#[derive(Debug, Hash)]
pub struct Context(pub cl_context);

impl Context {
    pub fn release(self) {
        release_context(self);
    }
}


pub fn create_context(devices: &[Device]) -> Result<Context, OCLError> {
    let mut err = 0;
    let r = unsafe {clCreateContext(std::ptr::null(), devices.len() as u32, devices.as_ptr() as *const *mut c_void, std::ptr::null_mut(), std::ptr::null_mut(), &mut err)};
    if err != 0 {
        return Err(OCLError::with_kind(OCLErrorKind::from_value(err)));
    }
    Ok(Context(r))
}

fn release_context(context: Context) {
    unsafe {clReleaseContext(context.0)};
}

#[derive(Clone, Copy, Debug)]
pub struct CommandQueue(pub cl_command_queue);

impl CommandQueue {
    pub fn release(self) {
        release_command_queue(self);
    }
}

pub fn create_command_queue(context: &Context, device: Device) -> Result<CommandQueue, OCLError> {
    let mut err = 0;
    let r = unsafe {clCreateCommandQueue(context.0, device.0, 0, &mut err)};
    //error("clCreateCommandQueue", err);
    if err != 0 {
        return Err(OCLError::with_kind(OCLErrorKind::from_value(err)));
    }
    Ok(CommandQueue(r))
}

pub fn finish(cq: CommandQueue) {
    unsafe {clFinish(cq.0)};
}

fn release_command_queue(cq: CommandQueue) {
    unsafe {clReleaseCommandQueue(cq.0)};
}
#[derive(Debug, Clone, Copy)]
pub struct Event(pub cl_event);

impl Event {
     pub fn wait(self) -> Result<(), OCLError> {
        wait_for_event(self)
    }
    pub fn release(self) {
        release_event(self).unwrap();
    }
}

pub fn wait_for_event(event: Event) -> Result<(), OCLError> {
    let event_vec: Vec<Event> = vec![event];
    //event_vec.push(event);
    let value = unsafe {clWaitForEvents(1, event_vec.as_ptr() as *mut cl_event)};
    if value != 0 {
        return Err(OCLError::with_kind(OCLErrorKind::from_value(value)));
    }
    event.release();
    Ok(())
}

pub fn release_event(event: Event) -> Result<(), OCLError>{
    let value = unsafe {clReleaseEvent(event.0)};
    if value != 0 {
        return Err(OCLError::with_kind(OCLErrorKind::from_value(value)));
    }
    Ok(())
}

pub enum MemFlags {
    MemReadWrite = 1,
    MemWriteOnly = 1 << 1,
    MemReadOnly = 1 << 2,
    MemUseHostPtr = 1 << 3,
    MemAllocHostPtr = 1 << 4,
    MemCopyHostPtr = 1 << 5,
    MemHostWriteOnly = 1 << 7,
    MemHostReadOnly = 1 << 8,
    MemHostNoAccess = 1 << 9,
}

impl core::ops::BitOr for MemFlags {
    type Output = u64;

    fn bitor(self, rhs: Self) -> Self::Output {
        self as u64 | rhs as u64
    }
}


#[cfg(not(feature = "nocache"))]
#[derive(Clone, Debug, Hash, Copy, Eq, PartialEq,)]
pub struct Mem(pub cl_mem);


#[cfg(feature = "nocache")]
#[derive(Debug, Hash, Eq, PartialEq,)]
pub struct Mem(pub cl_mem, bool);

#[cfg(feature = "nocache")]
use crate::prelude::CLDevice;

impl Mem {
    pub fn release(&mut self) {
        release_mem_object(self.0).unwrap();
    }
    #[cfg(not(feature = "nocache"))]
    pub fn as_cloned(&self) -> Mem {
        Mem(self.0)
    }
    
    #[cfg(feature = "nocache")]
    pub fn as_cloned<T>(&self, tensor: &Tensor<T>, device: CLDevice) -> Result<Mem, OCLError> {

        let mem = create_buffer::<T>(&device.get_ctx(), MemFlags::MemReadWrite as u64, tensor.ts.size, None)?;
        enqueue_copy_buffer(&device.get_queue(), self, &mem, tensor.ts.size)?;
        Ok(mem)
    }
     
    #[cfg(feature = "nocache")]
    pub fn as_cloned_no_drop(&mut self) -> Mem {
        self.1 = false;
        Mem(self.0, true)
    }
    
}

 

#[cfg(feature = "nocache")]
impl Drop for Mem {
    fn drop(&mut self) {
        if self.1 {
            self.release()
        }
        
    }
}

//impl TMemory for Mem {}

//impl HashMapMemory for Mem {}

pub fn create_buffer<T>(context: &Context, flag: u64, size: usize, data: Option<&[T]>) -> Result<*mut c_void, OCLError>{
    let mut err = 0;
    let host_ptr = match data {
        Some(d) => {d.as_ptr() as cl_mem},
        None => std::ptr::null_mut(),
    };
    let r = unsafe {clCreateBuffer(context.0, flag as u64, size*core::mem::size_of::<T>(), host_ptr, &mut err)};
    //error("clCreateBuffer", err);
    if err != 0 {
        return Err(OCLError::with_kind(OCLErrorKind::from_value(err)));
    }
    #[cfg(not(feature = "nocache"))]
    return Ok(r);
    #[cfg(feature = "nocache")]
    Ok(Mem(r, true))
    
}

pub fn release_mem_object(ptr: *mut c_void) -> Result<(), OCLError>{
    let value = unsafe {clReleaseMemObject(ptr)};
    if value != 0 {
        return Err(OCLError::with_kind(OCLErrorKind::from_value(value)));
    }
    Ok(())
}

pub fn retain_mem_object(mem: Mem) -> Result<(), OCLError>{
    let value = unsafe {clRetainMemObject(mem.0)};
    if value != 0 {
        return Err(OCLError::with_kind(OCLErrorKind::from_value(value)));
    }
    Ok(())
}

pub fn enqueue_read_buffer<T>(cq: &CommandQueue, mem: &Mem, data: &mut [T], block: bool) -> Result<Event, OCLError> {
    let mut events = vec![std::ptr::null_mut();1];
    let value = unsafe {clEnqueueReadBuffer(cq.0, mem.0, block as u32, 0, data.len()*core::mem::size_of::<T>(), data.as_ptr() as *mut c_void, 0, std::ptr::null(), events.as_mut_ptr() as *mut cl_event)};
    if value != 0 {
        return Err(OCLError::with_kind(OCLErrorKind::from_value(value)));
    }
    Ok(Event(events[0]))

}

pub fn enqueue_write_buffer<T>(cq: &CommandQueue, mem: &mut Mem, data: &[T], block: bool) -> Result<Event, OCLError> {
    let mut events = vec![std::ptr::null_mut();1];
    
    let value = unsafe {clEnqueueWriteBuffer(cq.0, mem.0, block as u32, 0, data.len()*core::mem::size_of::<T>(), data.as_ptr() as *mut c_void, 0, std::ptr::null(), events.as_mut_ptr() as *mut cl_event)};
    if value != 0 {
        return Err(OCLError::with_kind(OCLErrorKind::from_value(value)));
    }
    Ok(Event(events[0]))

}

pub fn enqueue_copy_buffer(cq: &CommandQueue, src_mem: &Mem, dst_mem: &Mem, size: usize) -> Result<(), OCLError>{
    let mut events = vec![std::ptr::null_mut();1];
    let value = unsafe {clEnqueueCopyBuffer(cq.0, src_mem.0, dst_mem.0, 0, 0, size*4, 0, std::ptr::null(), events.as_mut_ptr() as *mut cl_event)};
    if value != 0 {
        return Err(OCLError::with_kind(OCLErrorKind::from_value(value)));
    }
    wait_for_event(Event(events[0]))
    
}

pub fn enqueue_map_buffer() {
    
}
/*
pub fn enqueue_fill_buffer<T>(cq: &CommandQueue, mem: &Mem, pattern: Vec<T>) -> Event {
    let mut events = vec![std::ptr::null_mut();1];
    let offset = 0;
    let pattern_size = core::mem::size_of::<T>();
    let size = pattern_size*pattern.len();
    let err = unsafe {clEnqueueFillBuffer(cq.0, mem.0, pattern.as_ptr() as *mut c_void, pattern_size, offset, size, 0, std::ptr::null(), events.as_mut_ptr() as *mut cl_event)};
    println!("err enq copy bff: {}", err);
    Event(events[0])
}
*/
pub struct Program(pub cl_program);

impl Program {
    pub fn release(&mut self) {
        release_program(self).unwrap();
    }
}

enum ProgramInfo {
    BinarySizes = 0x1165,
    Binaries =    0x1166
}

enum ProgramBuildInfo {
    Status    = 0x1181,
    BuildLog = 0x1183
}

pub fn release_program(program: &mut Program) -> Result<(), OCLError>{
    let value = unsafe {clReleaseProgram(program.0)};
    if value != 0 {
        return Err(OCLError::with_kind(OCLErrorKind::from_value(value)));
    }
    Ok(())
}

pub fn create_program_with_source(context: &Context, src: &str) -> Result<Program, OCLError> {
    let mut err = 0;
    let cs = CString::new(src).expect("No cstring for you!");
    let lens = vec![cs.as_bytes().len()];
    let cstring: Vec<*const _> = vec![cs.as_ptr()];
    let r = unsafe {clCreateProgramWithSource(context.0, 1, cstring.as_ptr() as *const *const _, lens.as_ptr() as *const usize, &mut err)};
    if err != 0 {
        return Err(OCLError::with_kind(OCLErrorKind::from_value(err)));
    }
    Ok(Program(r))
}

pub fn build_program(program: &Program, devices: &[Device], options: Option<&str>) -> Result<(), OCLError> {
    let len = devices.len();

    let err;
    if let Some(x) = options {
        let cstring = CString::new(x).unwrap();
        err = unsafe {clBuildProgram(program.0, len as u32, devices.as_ptr() as *const *mut c_void, cstring.as_ptr(), std::ptr::null_mut(), std::ptr::null_mut())};

    } else {
        err = unsafe {clBuildProgram(program.0, len as u32, devices.as_ptr() as *const *mut c_void, std::ptr::null(), std::ptr::null_mut(), std::ptr::null_mut())};
    }
    if err != 0 {
        return Err(OCLError::with_kind(OCLErrorKind::from_value(err)));
    }
    Ok(())
    
}


#[derive(Debug, Clone)]
pub struct Kernel(pub cl_kernel);

impl Kernel {
    pub fn release(&mut self) {
        release_kernel(self).unwrap();
    }
}
pub fn create_kernel(program: &Program, str: &str) -> Result<Kernel, OCLError> {
    let mut err = 0;
    let cstring = CString::new(str).unwrap();
    let kernel = unsafe { clCreateKernel(program.0, cstring.as_ptr(),&mut err)};
    if err != 0 {
        return Err(OCLError::with_kind(OCLErrorKind::from_value(err)));
    }
    Ok(Kernel(kernel))
}
pub fn create_kernels_in_program(program: &Program) -> Result<Vec<Kernel>, OCLError> {
    let mut n_kernels: u32 = 0;
    let value = unsafe {clCreateKernelsInProgram(program.0, 0, std::ptr::null_mut(), &mut n_kernels)};
    if value != 0 {
        return Err(OCLError::with_kind(OCLErrorKind::from_value(value)));
    }

    let mut vec: Vec<usize> = vec![0; n_kernels as usize];
    let (ptr, len, cap) = (vec.as_mut_ptr(), vec.len(), vec.capacity());

    let mut kernels: Vec<Kernel> = unsafe {
        core::mem::forget(vec);
        Vec::from_raw_parts(ptr as *mut Kernel, len, cap)
    };
    let value = unsafe {clCreateKernelsInProgram(program.0, n_kernels, kernels.as_mut_ptr() as *mut cl_kernel, std::ptr::null_mut())};
    if value != 0 {
        return Err(OCLError::with_kind(OCLErrorKind::from_value(value)));
    }
    Ok(kernels)
}

pub fn release_kernel(kernel: &mut Kernel) -> Result<(), OCLError>{
    let value = unsafe {clReleaseKernel(kernel.0)};
    if value != 0 {
        return Err(OCLError::with_kind(OCLErrorKind::from_value(value)));
    }
    Ok(())
}

pub fn set_kernel_arg<T>(kernel: &Kernel, index: usize, arg: &T) {
    let value = unsafe {clSetKernelArg(kernel.0, index as u32, core::mem::size_of::<T>(), arg as *const T as *const c_void)};
    if value != 0 {
        OCLError::with_kind(OCLErrorKind::from_value(value));
    }
}
/* 
pub fn set_kernel_arg_c(kernel: &Kernel, index: usize, arg: *const c_void, size: usize) {
    error("clSetKernelArg", unsafe {clSetKernelArg(kernel.0, index as u32, size, arg)});
}
*/
pub fn enqueue_nd_range_kernel(cq: &CommandQueue, kernel: &Kernel, wd: usize, gws: &[usize; 3], lws: Option<&[usize;3]>, offset: Option<[usize; 3]>) -> Result<(), OCLError> {
    let mut events = vec![std::ptr::null_mut();1];
    let lws = match lws {
        Some(lws) => lws.as_ptr(),
        None => std::ptr::null()
    };
    let offset = match offset {
        Some(offset) => offset.as_ptr(),
        None => std::ptr::null()
    };

    let value = unsafe {clEnqueueNDRangeKernel(cq.0, kernel.0, wd as u32, offset, gws.as_ptr(), lws, 0, std::ptr::null(), events.as_mut_ptr() as *mut cl_event)};
    if value != 0 {
        return Err(OCLError::with_kind(OCLErrorKind::from_value(value)));
    }
    let e = Event(events[0]);
    wait_for_event(e)
}


