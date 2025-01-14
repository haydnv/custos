fn main() {
    // TODO: execute other opencl test to know whether opencl can actually be used

    if std::env::var("DOCS_RS").is_ok() {
        return;
    }

    #[cfg(not(docsrs))]
    #[cfg(feature = "opencl")]
    if has_device_unified_mem() {
        println!("cargo:rustc-cfg=unified_cl");
    }
}

#[cfg(not(docsrs))]
#[cfg(feature = "opencl")]
fn has_device_unified_mem() -> bool {
    println!("cargo:rerun-if-env-changed=CUSTOS_CL_DEVICE_IDX");
    println!("cargo:rerun-if-env-changed=CUSTOS_CU_DEVICE_IDX");
    println!("cargo:rerun-if-env-changed=CUSTOS_USE_UNIFIED");

    let device_idx = std::env::var("CUSTOS_CL_DEVICE_IDX")
        .unwrap_or_else(|_| "0".into())
        .parse::<usize>()
        .expect("Value in variable 'CUSTOS_CL_DEVICE_IDX' must be a usize value.");

    // this environment variable (CUSTOS_USE_UNIFIED) is used to either:
    // ... disable unified memory on unified memory devices, or
    // ... activate unified memory on devices with dedicated memory to check if
    // the code would compile on a device with unified memory.
    if let Ok(value) = std::env::var("CUSTOS_USE_UNIFIED") {
        if &value.to_ascii_lowercase() != "default" {
            let force_unified_mem = value.parse()
                .expect("'CUSTOS_USE_UNIFIED' must be either true, false or default. 
                    [
                        default=it is checked whether the device can use unified memory automatically.
                        true='simulates' unified memory to know if your code would compile on a device with unified memory.
                        false=deactivates unified memory
                    ]");
            if force_unified_mem {
                println!("Device forcefully uses unified memory!")
            } else {
                println!("Device won't use unified memory!")
            }
            return force_unified_mem;
        }
    }

    min_cl::CLDevice::new(device_idx)
        .unwrap_or_else(|_| panic!("Could not get an OpenCL device (at index {device_idx})."))
        .unified_mem
}
