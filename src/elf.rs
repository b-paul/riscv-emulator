#![allow(dead_code)]

use nom::{
    bytes::complete::{tag, take},
    combinator::map_opt,
    IResult
};

pub type Elf64Addr = usize;

pub type Elf64Offset = usize;

fn u16_parser(input: &[u8]) -> IResult<&[u8], u16> {
    map_opt(take(2usize), |bytes: &[u8]| match bytes {
        b if b.len() == 2 => {
            let mut bytes = [0; 2];
            bytes.copy_from_slice(b);
            Some(u16::from_le_bytes(bytes))
        }
        _ => None,
    })(input)
}

fn u32_parser(input: &[u8]) -> IResult<&[u8], u32> {
    map_opt(take(4usize), |bytes: &[u8]| match bytes {
        b if b.len() == 4 => {
            let mut bytes = [0; 4];
            bytes.copy_from_slice(b);
            Some(u32::from_le_bytes(bytes))
        }
        _ => None,
    })(input)
}

fn usize_parser(input: &[u8]) -> IResult<&[u8], usize> {
    map_opt(take(8usize), |bytes: &[u8]| match bytes {
        b if b.len() == 8 => {
            let mut bytes = [0; 8];
            bytes.copy_from_slice(b);
            Some(usize::from_le_bytes(bytes))
        }
        _ => None,
    })(input)
}

pub enum EiData {
    ElfDataNone,
    ElfData2Lsb,
    ElfData2Msb,
}

fn ei_data_parser(input: &[u8]) -> IResult<&[u8], EiData> {
    map_opt(take(1usize), |bytes: &[u8]| match bytes {
        &[0] => Some(EiData::ElfDataNone),
        &[1] => Some(EiData::ElfData2Lsb),
        &[2] => Some(EiData::ElfData2Msb),
        _ => None,
    })(input)
}

pub enum EiOsAbi {
    ElfOsAbiNone,
    ElfOsAbiHpUx,
    ElfOsAbiNetBSD,
    ElfOsAbiLinux,
    ElfOsAbiSolaris,
    ElfOsAbiAix,
    ElfOsAbiIrix,
    ElfOsAbiFreeBSD,
    ElfOsAbiTru64,
    ElfOsAbiModesto,
    ElfOsAbiOpenBSD,
    ElfOsAbiArm,
    ElfOsAbiStandalone,
}

fn ei_os_abi_parser(input: &[u8]) -> IResult<&[u8], EiOsAbi> {
    map_opt(take(1usize), |bytes: &[u8]| match bytes {
        &[0] => Some(EiOsAbi::ElfOsAbiNone),
        &[1] => Some(EiOsAbi::ElfOsAbiHpUx),
        &[2] => Some(EiOsAbi::ElfOsAbiNetBSD),
        &[3] => Some(EiOsAbi::ElfOsAbiLinux),
        &[6] => Some(EiOsAbi::ElfOsAbiSolaris),
        &[7] => Some(EiOsAbi::ElfOsAbiAix),
        &[8] => Some(EiOsAbi::ElfOsAbiIrix),
        &[9] => Some(EiOsAbi::ElfOsAbiFreeBSD),
        &[10] => Some(EiOsAbi::ElfOsAbiTru64),
        &[11] => Some(EiOsAbi::ElfOsAbiModesto),
        &[12] => Some(EiOsAbi::ElfOsAbiOpenBSD),
        &[97] => Some(EiOsAbi::ElfOsAbiArm),
        &[255] => Some(EiOsAbi::ElfOsAbiStandalone),
        _ => None,
    })(input)
}

pub struct EiOsAbiVersion(u8);

fn ei_os_abi_version_parser(input: &[u8]) -> IResult<&[u8], EiOsAbiVersion> {
    map_opt(take(1usize), |bytes: &[u8]| match bytes {
        &[b] => Some(EiOsAbiVersion(b)),
        _ => None,
    })(input)
}

pub enum EType {
    None,
    Rel,
    Exec,
    Dyn,
    Core,
}

fn e_type_parser(input: &[u8]) -> IResult<&[u8], EType> {
    // I'm just going to assume little endian for now......
    map_opt(take(2usize), |bytes: &[u8]| match bytes {
        &[0x00, 0x00] => Some(EType::None),
        &[0x01, 0x00] => Some(EType::Rel),
        &[0x02, 0x00] => Some(EType::Exec),
        &[0x03, 0x00] => Some(EType::Dyn),
        &[0x04, 0x00] => Some(EType::Core),
        &[0x05, 0x00] => Some(EType::None),
        _ => None,
    })(input)
}

pub struct Elf64Header {
    ei_data: EiData,
    ei_osabi: EiOsAbi,
    ei_abiversion: EiOsAbiVersion,
    e_type: EType,
    e_entry: Elf64Addr,
    e_phoff: Elf64Offset,
    e_shoff: Elf64Offset,
    e_flags: u32,
    e_ehsize: u16,
    e_phentsize: u16,
    e_phnum: u16,
    e_shentsize: u16,
    e_shnum: u16,
    e_shstrndx: u16,
}

/// Parses an Elf64_Ehdr in little endian
pub fn parse_elf64_header(input: &[u8]) -> IResult<&[u8], Elf64Header> {
    let (input, _) = tag(b"\x7fELF")(input)?;
    let (input, _) = tag(&[2])(input)?; // EI_CLASS as ELFCLASS64
    let (input, ei_data) = ei_data_parser(input)?;
    let (input, _) = take(1usize)(input)?; // EI_VERSION
    let (input, ei_osabi) = ei_os_abi_parser(input)?;
    let (input, ei_abiversion) = ei_os_abi_version_parser(input)?;
    let (input, _) = take(7usize)(input)?; // EI_PAD
    let (input, e_type) = e_type_parser(input)?;
    let (input, _) = tag(&[0xf3, 0x00])(input)?; // e_machine == EM_RISCV
    let (input, _) = tag(&[0x01, 0x00])(input)?; // e_version == EV_CURENT
    let (input, e_entry) = usize_parser(input)?;
    let (input, e_phoff) = usize_parser(input)?;
    let (input, e_shoff) = usize_parser(input)?;
    let (input, e_flags) = u32_parser(input)?;
    let (input, e_ehsize) = u16_parser(input)?;
    let (input, e_phentsize) = u16_parser(input)?;
    let (input, e_phnum) = u16_parser(input)?;
    let (input, e_shentsize) = u16_parser(input)?;
    let (input, e_shnum) = u16_parser(input)?;
    let (input, e_shstrndx) = u16_parser(input)?;

    Ok((
        input,
        Elf64Header {
            ei_data,
            ei_osabi,
            ei_abiversion,
            e_type,
            e_entry,
            e_phoff,
            e_shoff,
            e_flags,
            e_ehsize,
            e_phentsize,
            e_phnum,
            e_shentsize,
            e_shnum,
            e_shstrndx,
        },
    ))
}
