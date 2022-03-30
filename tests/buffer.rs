
use custos::{Buffer, Device, libs::{cpu::CPU, opencl::CLDevice}, VecRead, Error};

pub fn get_mut_slice<T>(buf: &mut Buffer<T>) -> &mut [T] {
    unsafe {
        std::slice::from_raw_parts_mut(buf.ptr, buf.len)
    }
}

pub fn get_slice<T>(buf: &Buffer<T>) -> &[T] {
    unsafe {
        std::slice::from_raw_parts(buf.ptr, buf.len)
    }
}

pub fn read<T, D: Device<T>>(device: &D, buf: Buffer<T>) -> Vec<T> where D: VecRead<T> {
    device.read(buf)
}

#[test]
fn test_cldevice_name() -> Result<(), Error> {
    let device = CLDevice::get(0)?;
    println!("{}", device.get_name()?);
    Ok(())
}

#[test]
fn test_cldevice_version() -> Result<(), Error> {
    let device = CLDevice::get(0)?;
    println!("{}", device.get_version()?);
    Ok(())
}

#[test]
fn test_cldevice_mem() -> Result<(), Error> {
    let device = CLDevice::get(0)?;
    println!("get_global_mem_size_in_gb: {}", device.get_global_mem_size_in_gb()?);
    println!("get_max_mem_alloc_in_gb: {}", device.get_max_mem_alloc_in_gb()?);
    Ok(())
}

#[test]
fn test_buffer_from_read() -> Result<(), Error> {
    let device = CLDevice::get(0)?;

    let buf = Buffer::<f32>::from((&device, [3.13, 3., 1., 8.]));
    assert_eq!(read(&device, buf), vec![3.13, 3., 1., 8.,]);

    let device = CPU::new();

    let buf = Buffer::<f32>::from((&device, [3.13, 3., 1., 8.]));
    assert_eq!(read(&device, buf), vec![3.13, 3., 1., 8.,]);
    Ok(())
}

#[test]
fn test_buffer_alloc_and_read() -> Result<(), Error> {
    let device = CPU::new();

    let mut buf = Buffer::<u8>::new(&device, 10);
    
    let buf_slice = get_mut_slice(&mut buf);
    buf_slice.copy_from_slice(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
    assert_eq!(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9], buf_slice);
    
    let cl = CLDevice::get(0)?;

    let buf = Buffer::<f32>::from((&cl, [3.13, 3., 1., 8.]));
    let buf_read = read(&cl, buf);
    assert_eq!(&[3.13, 3., 1., 8.], buf_read.as_slice());

    let buf = Buffer::<f32>::from((&device, [3.13, 3., 1., 8.]));
    let buf_read = read(&device, buf);
    assert_eq!(&[3.13, 3., 1., 8.], buf_read.as_slice());

    let buf_read = get_slice(&buf);
    assert_eq!(&[3.13, 3., 1., 8.], buf_read);

    Ok(())   
}

#[test]
fn test_use_number() {
    let num = Box::into_raw(Box::new(10));

    let buffer = Buffer { ptr: num, len: 1};

    let num = unsafe {
        Box::from_raw(buffer.ptr)
    };

    assert_eq!(num, Box::new(10));
}