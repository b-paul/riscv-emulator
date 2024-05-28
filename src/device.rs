#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum AccessType {
    Write,
}

impl AccessType {
    pub fn can_write(&self) -> bool {
        match self {
            AccessType::Write => true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DeviceRegister {
    /// The address that this register can be accessed at in memory.
    pub addr: usize,
    /// The amount of bytes that this register stores.
    pub size: usize,
    /// The access type of this register
    pub access_type: AccessType,
}

/// A device which interfaces with the cpu through memory mapped io.
///
/// A device will provide which addresses correspond to readable and writable, along with the size
/// of the buffer that said address refers to.
pub trait Device {
    /// Returns the list of registers for this device.
    fn get_registers(&self) -> Vec<DeviceRegister>;

    fn read_bytes(&mut self, addr: usize, size: usize) -> Vec<u8>;
    fn write_bytes(&mut self, addr: usize, bytes: &[u8]);
}
