#[derive(Debug)]
pub enum GPUError {
    ADAPTERNOTFOUND(),
    REQUESTDEVICEERROR(wgpu::RequestDeviceError),
    RENDERINGERROR(),
    SHADERSNOTCOMPILED(),
    UNCOMPATIBLEIMAGESIZE((u32, u32), (u32, u32))
}