use std::time::Instant;

use custos::{
    prelude::Number,
    range,
    wgpu::{launch_shader, WGPU},
    Buffer, Device, OpenCL,
};

use super::ElementWise;

pub fn wgpu_element_wise<T: Number, const N: usize>(
    device: &WGPU,
    lhs: &Buffer<T, WGPU, N>,
    rhs: &Buffer<T, WGPU, N>,
    out: &mut Buffer<T, WGPU, N>,
    op: &str,
) {
    let src = format!(
        "@group(0)
        @binding(0)
        var<storage, read_write> a: array<{datatype}>;
        
        @group(0)
        @binding(1)
        var<storage, read_write> b: array<{datatype}>;

        @group(0)
        @binding(2)
        var<storage, read_write> out: array<{datatype}>;
        
        
        @compute
        @workgroup_size(1)
        fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {{
            out[global_id.x] = a[global_id.x] {op} b[global_id.x];
        }}
        ",
        datatype = std::any::type_name::<T>()
    );

    launch_shader(device, &src, [lhs.len as u32, 1, 1], &[lhs, rhs, out]);
}

impl<T: Number, const N: usize> ElementWise<T, WGPU, N> for WGPU {
    #[inline]
    fn add(&self, lhs: &Buffer<T, WGPU, N>, rhs: &Buffer<T, WGPU, N>) -> Buffer<T, WGPU, N> {
        let mut out = self.retrieve(lhs.len, (lhs, rhs));
        wgpu_element_wise(self, lhs, rhs, &mut out, "+");
        out
    }

    #[inline]
    fn mul(&self, lhs: &Buffer<T, WGPU, N>, rhs: &Buffer<T, WGPU, N>) -> Buffer<T, WGPU, N> {
        let mut out = self.retrieve(lhs.len, (lhs, rhs));
        wgpu_element_wise(self, lhs, rhs, &mut out, "*");
        out
    }
}

#[test]
fn test_add() {
    let device = WGPU::new(wgpu::Backends::all()).unwrap();
    let lhs = Buffer::<f32, _>::from((&device, &[1., 2., 3., 4., -9.]));
    let rhs = Buffer::<f32, _>::from((&device, &[1., 2., 3., 4., -9.]));

    for _ in 0..1 {
        let out = device.add(&lhs, &rhs);
    }

    //   println!("read: {:?}", out.read());
}

#[test]
fn test_add_large() {
    const N: usize = 65535;

    let rhs_data = (0..N)
        .into_iter()
        .map(|val| val as f32)
        .collect::<Vec<f32>>();
    let out_actual_data = (0..N)
        .into_iter()
        .map(|val| val as f32 + 1.)
        .collect::<Vec<f32>>();

    let device = WGPU::new(wgpu::Backends::all()).unwrap();

    let lhs = Buffer::<f32, _>::from((&device, &[1.; N]));
    let rhs = Buffer::<f32, _>::from((&device, &rhs_data));

    let start = Instant::now();

    for _ in range(0..100) {
        let out = device.add(&lhs, &rhs);
        assert_eq!(out.read(), out_actual_data);
    }

    println!("wgpu dur: {:?}", start.elapsed());

    let device = OpenCL::new(0).unwrap();

    let lhs = Buffer::<f32, _>::from((&device, &[1.; N]));
    let rhs = Buffer::<f32, _>::from((&device, &rhs_data));

    let start = Instant::now();
    for _ in range(0..100) {
        let out = device.add(&lhs, &rhs);
        assert_eq!(out.read(), out_actual_data);
    }

    println!("ocl dur: {:?}", start.elapsed());

    //   println!("read: {:?}", out.read());
}