use raw_cpuid::CpuId;

pub fn init() {
    println!("{:x}", unsafe {
        x86_64::registers::model_specific::Msr::new(0x1b).read()
    });

    let cpuid = CpuId::new();

    println!(
        "CPU Vendor: {}",
        cpuid
            .get_vendor_info()
            .as_ref()
            .map_or_else(|| "unknown", |vf| vf.as_string(),)
    );

    println!(
        "CPU Model: {}",
        cpuid.get_extended_function_info().as_ref().map_or_else(
            || "n/a",
            |extfuninfo| extfuninfo.processor_brand_string().unwrap_or("unreadable"),
        )
    );

    cpuid.get_feature_info().as_ref().map_or_else(
        || println!("Family: n/a, Extended Family: n/a, Model: n/a, Extended Model: n/a, Stepping: n/a, Brand Index: n/a"),
        |finfo| {
            println!(
                "Family: {}, Extended Family: {}, Model: {}, Extended Model: {}, Stepping: {}, Brand Index: {}",
                finfo.family_id(),
                finfo.extended_family_id(),
                finfo.model_id(),
                finfo.extended_model_id(),
                finfo.stepping_id(),
                finfo.brand_index(),
            );
        },
    );
}
