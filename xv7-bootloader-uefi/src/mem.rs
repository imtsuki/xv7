use uefi::prelude::*;
use uefi::Result;

#[allow(unused)]
pub fn memory_map(services: &BootServices) -> Result<()> {
    let map_size = services.memory_map_size();
    let mut buf = vec![0u8; map_size];
    let (_, map_iter) = services.memory_map(&mut buf).log_warning()?;

    info!("map_iter size = {}", map_iter.len());

    for desc in map_iter {
        info!("{:?}", desc);
    }

    Ok(().into())
}
