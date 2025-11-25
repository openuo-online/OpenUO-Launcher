use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

/// 从 PE 文件（.exe 或 .dll）中读取版本信息
pub fn read_pe_version(path: &Path) -> Option<String> {
    let mut file = File::open(path).ok()?;
    
    // 读取 DOS 头
    let mut dos_header = [0u8; 64];
    file.read_exact(&mut dos_header).ok()?;
    
    // 检查 DOS 签名 "MZ"
    if &dos_header[0..2] != b"MZ" {
        return None;
    }
    
    // 获取 PE 头偏移（在 DOS 头的 0x3C 位置）
    let pe_offset = u32::from_le_bytes([
        dos_header[0x3C],
        dos_header[0x3D],
        dos_header[0x3E],
        dos_header[0x3F],
    ]);
    
    // 跳转到 PE 头
    file.seek(SeekFrom::Start(pe_offset as u64)).ok()?;
    
    // 读取 PE 签名
    let mut pe_sig = [0u8; 4];
    file.read_exact(&mut pe_sig).ok()?;
    
    // 检查 PE 签名 "PE\0\0"
    if &pe_sig != b"PE\0\0" {
        return None;
    }
    
    // 读取 COFF 文件头
    let mut coff_header = [0u8; 20];
    file.read_exact(&mut coff_header).ok()?;
    
    // 获取可选头大小
    let optional_header_size = u16::from_le_bytes([coff_header[16], coff_header[17]]);
    
    if optional_header_size < 96 {
        return None;
    }
    
    // 读取可选头的前 96 字节
    let mut optional_header = vec![0u8; optional_header_size as usize];
    file.read_exact(&mut optional_header).ok()?;
    
    // 检查魔数（PE32 或 PE32+）
    let magic = u16::from_le_bytes([optional_header[0], optional_header[1]]);
    let is_pe32_plus = magic == 0x20b;
    
    // 获取数据目录的数量和资源表位置
    let num_rva_offset = if is_pe32_plus { 108 } else { 92 };
    if optional_header.len() < num_rva_offset + 4 {
        return None;
    }
    
    let resource_dir_offset = if is_pe32_plus { 112 + 16 } else { 96 + 16 };
    if optional_header.len() < resource_dir_offset + 8 {
        return None;
    }
    
    // 获取资源表的 RVA 和大小
    let resource_rva = u32::from_le_bytes([
        optional_header[resource_dir_offset],
        optional_header[resource_dir_offset + 1],
        optional_header[resource_dir_offset + 2],
        optional_header[resource_dir_offset + 3],
    ]);
    
    if resource_rva == 0 {
        return None;
    }
    
    // 读取节表来找到资源节
    let section_header_offset = pe_offset as usize + 24 + optional_header_size as usize;
    let num_sections = u16::from_le_bytes([coff_header[2], coff_header[3]]);
    
    file.seek(SeekFrom::Start(section_header_offset as u64)).ok()?;
    
    let mut resource_section_offset = 0u32;
    let mut resource_section_rva = 0u32;
    
    for _ in 0..num_sections {
        let mut section_header = [0u8; 40];
        file.read_exact(&mut section_header).ok()?;
        
        let virtual_address = u32::from_le_bytes([
            section_header[12],
            section_header[13],
            section_header[14],
            section_header[15],
        ]);
        
        let virtual_size = u32::from_le_bytes([
            section_header[8],
            section_header[9],
            section_header[10],
            section_header[11],
        ]);
        
        let raw_data_offset = u32::from_le_bytes([
            section_header[20],
            section_header[21],
            section_header[22],
            section_header[23],
        ]);
        
        // 检查资源 RVA 是否在这个节中
        if resource_rva >= virtual_address && resource_rva < virtual_address + virtual_size {
            resource_section_offset = raw_data_offset;
            resource_section_rva = virtual_address;
            break;
        }
    }
    
    if resource_section_offset == 0 {
        return None;
    }
    
    // 计算资源表在文件中的实际偏移
    let resource_file_offset = resource_section_offset + (resource_rva - resource_section_rva);
    
    // 尝试查找 VS_VERSION_INFO 资源
    // 这里使用简化的方法：直接搜索 VS_FIXEDFILEINFO 结构
    file.seek(SeekFrom::Start(resource_file_offset as u64)).ok()?;
    
    let mut resource_data = vec![0u8; 4096.min(file.metadata().ok()?.len() as usize - resource_file_offset as usize)];
    file.read_exact(&mut resource_data).ok()?;
    
    // 搜索 VS_FIXEDFILEINFO 签名 0xFEEF04BD
    for i in 0..resource_data.len().saturating_sub(52) {
        let signature = u32::from_le_bytes([
            resource_data[i],
            resource_data[i + 1],
            resource_data[i + 2],
            resource_data[i + 3],
        ]);
        
        if signature == 0xFEEF04BD {
            // 找到了 VS_FIXEDFILEINFO
            // 文件版本在偏移 8-15 字节
            if i + 16 <= resource_data.len() {
                let file_version_ms = u32::from_le_bytes([
                    resource_data[i + 8],
                    resource_data[i + 9],
                    resource_data[i + 10],
                    resource_data[i + 11],
                ]);
                
                let file_version_ls = u32::from_le_bytes([
                    resource_data[i + 12],
                    resource_data[i + 13],
                    resource_data[i + 14],
                    resource_data[i + 15],
                ]);
                
                let major = (file_version_ms >> 16) & 0xFFFF;
                let minor = file_version_ms & 0xFFFF;
                let build = (file_version_ls >> 16) & 0xFFFF;
                let revision = file_version_ls & 0xFFFF;
                
                // 返回版本号，通常只显示前三部分
                if revision == 0 {
                    return Some(format!("{}.{}.{}", major, minor, build));
                } else {
                    return Some(format!("{}.{}.{}.{}", major, minor, build, revision));
                }
            }
        }
    }
    
    None
}
