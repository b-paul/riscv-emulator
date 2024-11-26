// TODO
// Create a constants file
// Give error messages to errors
// Finish writing docs for everything

use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Elf {
    /// Mapping from symbol name to offset
    // TODO is taking an owned String necessary?
    symbol_table: HashMap<String, Symbol>,
    /// Entry point into the program (as an address)
    entry_point: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Symbol {
    pub value: usize,
    pub size: usize,
    // TODO the rest
}

impl Elf {
    pub fn new(bin: &[u8]) -> Result<Elf, ElfParseError> {
        let read_elf = ReadElf::read_elf(bin)?;

        let sym_table_idx = read_elf
            .find_sh_index(".symtab")
            .ok_or(ElfParseError::NoStringTable)?;
        let str_table_idx = read_elf
            .find_sh_index(".strtab")
            .ok_or(ElfParseError::NoStringTable)?;

        let read_symbol_table = match &read_elf.section_headers[sym_table_idx].contents {
            SectionHeaderContents::SymbolTable(vec) => vec,
            _ => return Err(ElfParseError::InvalidSectionHeader),
        };

        let symbol_table = read_symbol_table
            .iter()
            .map(|entry| {
                let name = read_elf
                    .read_str_at(entry.name as usize, str_table_idx)
                    .map_err(|e| ElfParseError::Utf8Error(e))?
                    .to_owned();
                let symbol = Symbol {
                    value: entry.value,
                    size: entry.size,
                };
                Ok((name, symbol))
            })
            .collect::<Result<_, _>>()?;

        Ok(Elf {
            symbol_table,
            entry_point: read_elf.header.entry,
        })
    }

    /// Returns the entry point of the program
    pub fn get_entry(&self) -> usize {
        self.entry_point
    }

    /// Get the symbol with the corresponding name `name`, if it exists
    pub fn get_symbol(&self, name: &str) -> Option<&Symbol> {
        self.symbol_table.get(name)
    }
}

/// Contains all (relevant) information that can be obtained from the ELF header of a binary.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct ReadElf {
    header: ElfHeader,
    program_headers: Vec<ProgramHeader>,
    section_headers: Vec<SectionHeader>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElfParseError {
    InvalidIdentifier,
    InvalidProgramHeader,
    InvalidSectionHeader,
    TooSmall,
    Unsupported,
    NotExecutable,
    InvalidAddressSize,
    NoStringTable,
    Utf8Error(std::str::Utf8Error),
}

impl ReadElf {
    pub fn read_elf(bin: &[u8]) -> Result<ReadElf, ElfParseError> {
        let header = ElfHeader::read_elf_header(bin)?;
        let program_headers = ProgramHeader::read_program_headers(bin, &header)?;
        let section_headers = SectionHeader::read_section_headers(bin, &header)?;

        if !matches!(
            section_headers[header.sh_str_table_idx].contents,
            SectionHeaderContents::StringTable(_)
        ) {
            return Err(ElfParseError::InvalidSectionHeader);
        }

        Ok(ReadElf {
            header,
            program_headers,
            section_headers,
        })
    }

    fn find_sh_index(&self, header: &str) -> Option<usize> {
        self.section_headers.iter().position(|sh| {
            self.read_str_at(sh.name as usize, self.header.sh_str_table_idx) == Ok(header)
        })
    }

    /// Read the string at index `idx` in the string table at index table_idx.
    fn read_str_at(&self, idx: usize, table_idx: usize) -> Result<&str, std::str::Utf8Error> {
        match &self.section_headers[table_idx].contents {
            SectionHeaderContents::StringTable(vec) => {
                let str = &vec[idx..];
                let end = str
                    .iter()
                    .position(|&b| b == 0)
                    .expect("Invalid Elf created");
                std::str::from_utf8(&str[..end])
            }
            // TODO this should probably be an error case
            _ => panic!("Invalid table_idx passed"),
        }
    }
}

fn read_u8(bytes: &[u8]) -> Result<(&[u8], u8), ElfParseError> {
    if bytes.len() < 1 {
        return Err(ElfParseError::TooSmall);
    }
    Ok((&bytes[1..], bytes[0]))
}

fn read_u16(bytes: &[u8]) -> Result<(&[u8], u16), ElfParseError> {
    let mut buf = [0; 2];
    if bytes.len() < 2 {
        return Err(ElfParseError::TooSmall);
    }
    buf.copy_from_slice(&bytes[0..2]);
    Ok((&bytes[2..], u16::from_le_bytes(buf)))
}

fn read_u32(bytes: &[u8]) -> Result<(&[u8], u32), ElfParseError> {
    let mut buf = [0; 4];
    if bytes.len() < 4 {
        return Err(ElfParseError::TooSmall);
    }
    buf.copy_from_slice(&bytes[0..4]);
    Ok((&bytes[4..], u32::from_le_bytes(buf)))
}

fn read_u64(bytes: &[u8]) -> Result<(&[u8], u64), ElfParseError> {
    let mut buf = [0; 8];
    if bytes.len() < 8 {
        return Err(ElfParseError::TooSmall);
    }
    buf.copy_from_slice(&bytes[0..8]);
    Ok((&bytes[8..], u64::from_le_bytes(buf)))
}

/// Contains information about the targetted machine, and gives offsets to other parts of the ELF.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct ElfHeader {
    /// Address of program entry
    entry: usize,
    /// Offset of the program hedder
    phoff: usize,
    /// Offset of the section header
    shoff: usize,
    /// Flags (TODO use them lol)
    flags: u32,
    /// The number of bytes in an entry of the program header table
    p_entry_size: usize,
    /// The number of program header table entries
    p_entry_count: usize,
    /// The number of bytes in an entry of the section header table
    s_entry_size: usize,
    /// The number of section header table entries
    s_entry_count: usize,
    /// THe index into the section header table corresponding to the section name string table.
    sh_str_table_idx: usize,
}

impl ElfHeader {
    /// Read the header of the ELF. Here we validate that the machine is indeed RISCV.
    fn read_elf_header(bin: &[u8]) -> Result<ElfHeader, ElfParseError> {
        let identifier = ElfHeaderIdentifier::read_elf_header_identifier(bin)?;

        // We don't support 32 bit atm (TODO?)
        if identifier.class != ElfClass::C64 {
            return Err(ElfParseError::Unsupported);
        }

        let bin = &bin[16..];

        let (bin, typ) = read_u16(&bin)?;
        let (bin, machine) = read_u16(&bin)?;
        let (bin, version) = read_u32(&bin)?;
        let (bin, entry) = read_u64(&bin)?;
        let (bin, phoff) = read_u64(&bin)?;
        let (bin, shoff) = read_u64(&bin)?;
        let (bin, flags) = read_u32(&bin)?;
        // TODO do i need this ?
        let (bin, _header_size) = read_u16(&bin)?;
        let (bin, p_entry_size) = read_u16(&bin)?;
        // TODO account for p_entry_count being PN_XNUM (more than u16::MAX)
        let (bin, p_entry_count) = read_u16(&bin)?;
        let (bin, s_entry_size) = read_u16(&bin)?;
        // TODO account for this being zero while we have a section header table
        let (bin, s_entry_count) = read_u16(&bin)?;
        // TODO account for this being SHN_XINDEX when the string table index is past 0xff00
        let (_bin, sh_str_table_idx) = read_u16(&bin)?;

        // Check the elf is an executable (ET_EXEL = 2)
        // TODO confirm we actually only want to support executables
        if typ != 2 {
            return Err(ElfParseError::NotExecutable);
        }

        // Check the elf is for a RISCV machine (EM_RISCV = 243)
        if machine != 243 {
            return Err(ElfParseError::Unsupported);
        }

        // version must be ET_CURRENT = 1
        if version != 1 {
            return Err(ElfParseError::Unsupported);
        }

        let entry = entry
            .try_into()
            .map_err(|_| ElfParseError::InvalidAddressSize)?;
        let phoff = phoff
            .try_into()
            .map_err(|_| ElfParseError::InvalidAddressSize)?;
        let shoff = shoff
            .try_into()
            .map_err(|_| ElfParseError::InvalidAddressSize)?;
        let p_entry_size = p_entry_size
            .try_into()
            .map_err(|_| ElfParseError::InvalidAddressSize)?;
        let p_entry_count = p_entry_count
            .try_into()
            .map_err(|_| ElfParseError::InvalidAddressSize)?;
        let s_entry_size = s_entry_size
            .try_into()
            .map_err(|_| ElfParseError::InvalidAddressSize)?;
        let s_entry_count = s_entry_count
            .try_into()
            .map_err(|_| ElfParseError::InvalidAddressSize)?;
        let sh_str_table_idx = sh_str_table_idx
            .try_into()
            .map_err(|_| ElfParseError::InvalidAddressSize)?;

        Ok(ElfHeader {
            entry,
            phoff,
            shoff,
            flags,
            p_entry_size,
            p_entry_count,
            s_entry_size,
            s_entry_count,
            sh_str_table_idx,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct ElfHeaderIdentifier {
    class: ElfClass,
    osabi: ElfOsAbi,
    abi_version: u8,
}

impl ElfHeaderIdentifier {
    /// Reads and validates the `e_ident` field of the elf header.
    fn read_elf_header_identifier(bin: &[u8]) -> Result<ElfHeaderIdentifier, ElfParseError> {
        let identifier = &bin[0..16];

        if identifier[0..4] != [0x7f, b'E', b'L', b'F'] {
            return Err(ElfParseError::InvalidIdentifier);
        }

        let class = match identifier[4] {
            1 => ElfClass::C32,
            2 => ElfClass::C64,
            _ => return Err(ElfParseError::InvalidIdentifier),
        };

        // Must be little endian corresponding to ELFDATALSB=1
        if identifier[5] != 1 {
            return Err(ElfParseError::InvalidIdentifier);
        }

        // EV_CURRENT = 1
        if identifier[6] != 1 {
            return Err(ElfParseError::InvalidIdentifier);
        }

        // osabi must be ELFOSABI_RISCV which is
        let osabi = match identifier[7] {
            0 => ElfOsAbi::None,
            255 => ElfOsAbi::Standalone,
            _ => return Err(ElfParseError::InvalidIdentifier),
        };

        let abi_version = identifier[8];

        Ok(ElfHeaderIdentifier {
            class,
            osabi,
            abi_version,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum ElfClass {
    C32,
    C64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum ElfOsAbi {
    // TODO add more i guess, if there's a need ?!
    None,
    Standalone,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct ProgramHeader {
    typ: ProgramHeaderType,
    /// Read/Write/Execute flags
    flags: u32,
    offset: u64,
    vaddr: usize,
    paddr: usize,
    file_size: usize,
    mem_size: usize,
    align: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum ProgramHeaderType {
    Null,
    Load,
    Dynamic,
    Interpreter,
    Note,
    SharedLib,
    ProgramHeader,
    RiscvAttributes,
}

impl ProgramHeader {
    fn read_program_headers(
        bin: &[u8],
        header: &ElfHeader,
    ) -> Result<Vec<ProgramHeader>, ElfParseError> {
        (0..header.p_entry_count)
            .map(|i| {
                let header = &bin[header.phoff + i * header.p_entry_size..];
                let (header, typ) = read_u32(&header)?;
                let (header, flags) = read_u32(&header)?;
                let (header, offset) = read_u64(&header)?;
                let (header, vaddr) = read_u64(&header)?;
                let (header, paddr) = read_u64(&header)?;
                let (header, file_size) = read_u64(&header)?;
                let (header, mem_size) = read_u64(&header)?;
                let (_header, align) = read_u64(&header)?;

                let typ = match typ {
                    0 => Ok(ProgramHeaderType::Null),
                    1 => Ok(ProgramHeaderType::Load),
                    2 => Ok(ProgramHeaderType::Dynamic),
                    3 => Ok(ProgramHeaderType::Interpreter),
                    4 => Ok(ProgramHeaderType::Note),
                    5 => Ok(ProgramHeaderType::SharedLib),
                    6 => Ok(ProgramHeaderType::ProgramHeader),
                    0x70000003 => Ok(ProgramHeaderType::RiscvAttributes),
                    _ => Err(ElfParseError::InvalidProgramHeader),
                }?;
                let vaddr = vaddr
                    .try_into()
                    .map_err(|_| ElfParseError::InvalidAddressSize)?;
                let paddr = paddr
                    .try_into()
                    .map_err(|_| ElfParseError::InvalidAddressSize)?;
                let file_size = file_size
                    .try_into()
                    .map_err(|_| ElfParseError::InvalidAddressSize)?;
                let mem_size = mem_size
                    .try_into()
                    .map_err(|_| ElfParseError::InvalidAddressSize)?;
                let align = align
                    .try_into()
                    .map_err(|_| ElfParseError::InvalidAddressSize)?;

                Ok(ProgramHeader {
                    typ,
                    flags,
                    offset,
                    vaddr,
                    paddr,
                    file_size,
                    mem_size,
                    align,
                })
            })
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct SectionHeader {
    name: u32,
    typ: SectionHeaderType,
    flags: u64,
    addr: usize,
    offset: usize,
    size: usize,
    link: u32,
    info: u32,
    addralign: usize,
    entsize: usize,
    contents: SectionHeaderContents,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum SectionHeaderType {
    Null,
    ProgramBits,
    SymbolTable,
    StringTable,
    RelocationEntries,
    HashTable,
    Dynamic,
    Note,
    NoBits,
    RelocationOffsets,
    SharedLib,
    DynamicSymbols,
    RiscvAttributes,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum SectionHeaderContents {
    None,
    SymbolTable(Vec<SymbolTableEntry>),
    StringTable(Vec<u8>),
    // Rest of the types of tables are TODO
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct SymbolTableEntry {
    name: u32,
    info: u8,
    other: u8,
    /// Index into the section header table that is relevant to this entry
    sh_index: usize,
    value: usize,
    size: usize,
}

impl SectionHeader {
    fn read_section_headers(
        bin: &[u8],
        header: &ElfHeader,
    ) -> Result<Vec<SectionHeader>, ElfParseError> {
        (0..header.s_entry_count)
            .map(|i| {
                let header = &bin[header.shoff + i * header.s_entry_size..];
                let (header, name) = read_u32(&header)?;
                let (header, typ) = read_u32(&header)?;
                let (header, flags) = read_u64(&header)?;
                let (header, addr) = read_u64(&header)?;
                let (header, offset) = read_u64(&header)?;
                let (header, size) = read_u64(&header)?;
                let (header, link) = read_u32(&header)?;
                let (header, info) = read_u32(&header)?;
                let (header, addralign) = read_u64(&header)?;
                let (_header, entsize) = read_u64(&header)?;

                let typ = match typ {
                    0 => Ok(SectionHeaderType::Null),
                    1 => Ok(SectionHeaderType::ProgramBits),
                    2 => Ok(SectionHeaderType::SymbolTable),
                    3 => Ok(SectionHeaderType::StringTable),
                    4 => Ok(SectionHeaderType::RelocationEntries),
                    5 => Ok(SectionHeaderType::HashTable),
                    6 => Ok(SectionHeaderType::Dynamic),
                    7 => Ok(SectionHeaderType::Note),
                    8 => Ok(SectionHeaderType::NoBits),
                    9 => Ok(SectionHeaderType::RelocationOffsets),
                    10 => Ok(SectionHeaderType::SharedLib),
                    11 => Ok(SectionHeaderType::DynamicSymbols),
                    0x70000003 => Ok(SectionHeaderType::RiscvAttributes),
                    _ => Err(ElfParseError::InvalidSectionHeader),
                }?;
                let addr = addr
                    .try_into()
                    .map_err(|_| ElfParseError::InvalidAddressSize)?;
                let offset = offset
                    .try_into()
                    .map_err(|_| ElfParseError::InvalidAddressSize)?;
                let size = size
                    .try_into()
                    .map_err(|_| ElfParseError::InvalidAddressSize)?;
                let addralign = addralign
                    .try_into()
                    .map_err(|_| ElfParseError::InvalidAddressSize)?;
                let entsize = entsize
                    .try_into()
                    .map_err(|_| ElfParseError::InvalidAddressSize)?;

                let contents = match typ {
                    SectionHeaderType::Null => SectionHeaderContents::None,
                    SectionHeaderType::ProgramBits => SectionHeaderContents::None,
                    SectionHeaderType::SymbolTable => {
                        let table = &bin[offset..offset + size];
                        // TODO verify this mod check is correct
                        if entsize == 0 || size % entsize != 0 {
                            return Err(ElfParseError::InvalidSectionHeader);
                        }
                        let count = size / entsize;
                        SectionHeaderContents::SymbolTable(
                            (0..count)
                                .map(|i| {
                                    let entry = &table[i * entsize..];
                                    let (entry, name) = read_u32(&entry)?;
                                    let (entry, info) = read_u8(&entry)?;
                                    let (entry, other) = read_u8(&entry)?;
                                    let (entry, sh_index) = read_u16(&entry)?;
                                    let (entry, value) = read_u64(&entry)?;
                                    let (_entry, size) = read_u64(&entry)?;

                                    let sh_index = sh_index
                                        .try_into()
                                        .map_err(|_| ElfParseError::InvalidAddressSize)?;
                                    let value = value
                                        .try_into()
                                        .map_err(|_| ElfParseError::InvalidAddressSize)?;
                                    let size = size
                                        .try_into()
                                        .map_err(|_| ElfParseError::InvalidAddressSize)?;

                                    Ok(SymbolTableEntry {
                                        name,
                                        info,
                                        other,
                                        sh_index,
                                        value,
                                        size,
                                    })
                                })
                                .collect::<Result<Vec<_>, _>>()?,
                        )
                    }
                    SectionHeaderType::StringTable => {
                        if bin[offset..].len() < size {
                            return Err(ElfParseError::InvalidSectionHeader);
                        }
                        SectionHeaderContents::StringTable(bin[offset..offset + size].to_owned())
                    }
                    SectionHeaderType::RelocationEntries => SectionHeaderContents::None,
                    SectionHeaderType::HashTable => SectionHeaderContents::None,
                    SectionHeaderType::Dynamic => SectionHeaderContents::None,
                    SectionHeaderType::Note => SectionHeaderContents::None,
                    SectionHeaderType::NoBits => SectionHeaderContents::None,
                    SectionHeaderType::RelocationOffsets => SectionHeaderContents::None,
                    SectionHeaderType::SharedLib => SectionHeaderContents::None,
                    SectionHeaderType::DynamicSymbols => SectionHeaderContents::None,
                    SectionHeaderType::RiscvAttributes => SectionHeaderContents::None,
                };

                Ok(SectionHeader {
                    name,
                    typ,
                    flags,
                    addr,
                    offset,
                    size,
                    link,
                    info,
                    addralign,
                    entsize,
                    contents,
                })
            })
            .collect()
    }
}
