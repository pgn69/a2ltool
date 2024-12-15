use crate::{datatype, dwarf::DebugData, insert, search, BinFileFormat};
use a2lfile::{A2lFile, AddrType, ByteOrderEnum, CharacteristicType, DataType};
use bin_file::{BinFile, IHexFormat, SRecordAddressLength};
use std::{ffi::OsString, fs::File, io::Write, path::Path};

#[derive(Debug)]
struct Calibration {
    symbol: String,
    value_repr: Option<String>,
    address: Option<u32>,
    size: Option<u16>,
    dim: Option<u16>,
    dtype: Option<DataType>,
    endianess: ByteOrderEnum,
}

pub(crate) fn calibration_from_binary_to_csv(
    a2l_file: &mut A2lFile, 
    elf_info: &Option<DebugData>,
    enable_structures: bool,
    default_endianess: &ByteOrderEnum, 
    binfile: &BinFile, 
    csv_file: &OsString,
    log_msgs: &mut Vec<String>,
) -> Result<bool, String> {
    let mut calibrations = read_calibrations_csv(csv_file, &default_endianess);
    calibration_symbols_load(&mut calibrations, a2l_file, elf_info, enable_structures, log_msgs)?;
    read_calibration(&mut calibrations, &binfile, log_msgs)?;
    write_calibrations_csv(csv_file, &calibrations)?;
    Ok(true)
}

pub(crate) fn calibration_from_csv_to_binary(
    a2l_file: &mut A2lFile, 
    elf_info: &Option<DebugData>,
    enable_structures: bool,
    default_endianess: &ByteOrderEnum, 
    binfile: &mut BinFile, 
    csv_file: &OsString,
    binary_file: &OsString,
    log_msgs: &mut Vec<String>,
) -> Result<bool, String> {
    let mut calibrations = read_calibrations_csv(csv_file, &default_endianess);
    calibration_symbols_load(&mut calibrations, a2l_file, elf_info, enable_structures, log_msgs)?;
    write_calibration(&calibrations, binfile, log_msgs)?;
    save_binfile(binary_file, binfile, log_msgs)?;
    Ok(true)
}

pub(crate) fn guess_default_endianess(
    a2l_file: &A2lFile,
    elf_info: &Option<DebugData>,
) -> Option<ByteOrderEnum> {
    let mut default_order = None;
    for module in &a2l_file.project.module {
        if let Some(mod_common) = &module.mod_common {
            if let Some(byte_order) = &mod_common.byte_order {
                if default_order.is_none() {
                    default_order = Some(byte_order.byte_order.clone());
                } else if byte_order.byte_order != default_order.unwrap() {
                    panic!("Mixed BYTE_ORDER in MOD_COMMON not supported. Specify the --default_byte_order on the command line.")
                }
            }
        }
    }
    if default_order.is_none() {
        if let Some(debugdata) = elf_info {
            default_order = match debugdata.endian {
                object::Endianness::Little => Some(ByteOrderEnum::LittleEndian),
                object::Endianness::Big => Some(ByteOrderEnum::BigEndian),
            };
        }
    }
    default_order
}

fn read_calibrations_csv(
    csv_file: &OsString,
    default_endianess: &ByteOrderEnum,
) -> Vec<Calibration> {
    let mut ret: Vec<Calibration> = Vec::new();
    let text = std::fs::read_to_string(csv_file).expect("Cannot read CSV file");

    for line in text.lines() {
        let fields: Vec<&str> = line.split(';').collect();
        if fields.len() > 0 {
            let f = fields[0].trim();
            if f.len() > 0 && !f.starts_with("#") {
                let mut cal = Calibration {
                    symbol: f.to_string(),
                    value_repr: None,
                    address: None,
                    size: None,
                    dim: None,
                    dtype: None,
                    endianess: default_endianess.clone(),
                };
                if fields.len() > 1 {
                    let f = fields[1].trim();
                    if f.len() > 0 {
                        cal.value_repr = Some(f.to_string());
                    }
                }
                ret.push(cal);
            }
        }
    }

    ret
}

fn write_calibrations_csv(
    csv_file: &OsString,
    calibrations: &Vec<Calibration>,
) -> Result<bool, String> {
    let mut calmap = std::collections::HashMap::new();
    for cal in calibrations {
        calmap.insert(&cal.symbol[..], cal);
    }

    let text = std::fs::read_to_string(csv_file).expect("Cannot read CSV file");
    let mut file = File::create(csv_file).expect("Cannot open CSV file for writing");

    for line in text.lines() {
        let mut bypass = true;
        let l = line.trim();
        let fields: Vec<&str> = line.split(';').collect();
        if fields.len() > 0 {
            let f = fields[0].trim();
            if f.len() > 0 && !f.starts_with("#") {
                if let Some(cal) = calmap.get(f) {
                    bypass = false;
                    writeln!(
                        file,
                        "{};{}",
                        (*cal).symbol,
                        (*cal).value_repr.as_ref().unwrap_or(&String::from(""))
                    )
                    .expect("Error writing CSV file");
                }
            }
        }
        if bypass {
            writeln!(file, "{}", l).expect("Error writing CSV file");
        }
    }

    Ok(true)
}

fn calibration_symbols_load(
    calibrations: &mut Vec<Calibration>,
    a2l_file: &mut A2lFile,
    elf_info: &Option<DebugData>,
    enable_structures: bool,
    log_msgs: &mut Vec<String>,
) -> Result<bool, String> {
    let mut characteristics = search::search_characteristics(a2l_file, &[".*"], log_msgs);

    if let Some(debugdata) = &elf_info {
        // Add the characteristics that are listed in the CSV file, but not in the A2L.
        let mut characteristic_symbols: Vec<&str> = Vec::new();
        for cal in &*calibrations {
            if !characteristics.contains_key(&cal.symbol) {
                characteristic_symbols.push(&cal.symbol);
            }
        }
        if !characteristic_symbols.is_empty() {
            insert::insert_items(
                a2l_file,
                debugdata,
                vec![],
                characteristic_symbols,
                Some("AUTO"),
                log_msgs,
                enable_structures,
            );

            characteristics = search::search_characteristics(a2l_file, &[".*"], log_msgs);
        }
    }

    let record_layouts = search::search_reord_layout(a2l_file, &[".*"], log_msgs);

    for cal in &mut *calibrations {
        if let Some(characteristic) = characteristics.get(&cal.symbol) {
            cal.address = Some(characteristic.address);
            if characteristic.byte_order.is_some() {
                cal.endianess = characteristic.byte_order.as_ref().unwrap().byte_order;
            }
            match characteristic.characteristic_type {
                CharacteristicType::Value => {
                    cal.dim = Some(1);
                }
                CharacteristicType::ValBlk => {
                    if let Some(matrix_dim) = &characteristic.matrix_dim {
                        cal.dim = Some(matrix_dim.dim_list.iter().product());
                    } else {
                        log_msgs.push(format!(
                            "Characteristic {} matrix dimension not found",
                            &cal.symbol
                        ));
                        continue;
                    }
                }
                _ => {
                    log_msgs.push(format!(
                        "Characteristic {} type {} not supported",
                        &cal.symbol, &characteristic.characteristic_type
                    ));
                    continue;
                }
            }
            if let Some(rl) = record_layouts.get(&characteristic.deposit) {
                if let Some(fnc_value) = &rl.fnc_values {
                    if fnc_value.position == 1 && fnc_value.address_type == AddrType::Direct {
                        cal.size = Some(datatype::get_datatype_size(&fnc_value.datatype));
                        cal.dtype = Some(fnc_value.datatype);
                    } else {
                        log_msgs.push(format!(
                            "Characteristic {} record layout not supported",
                            &cal.symbol
                        ));
                    }
                } else {
                    log_msgs.push(format!(
                        "Characteristic {} data type not found",
                        &cal.symbol
                    ));
                };
            };
        } else {
            log_msgs.push(format!("Symbol {} not found", &cal.symbol));
        }
    }

    Ok(true)
}

fn read_calibration(
    calibrations: &mut Vec<Calibration>,
    binfile: &BinFile,
    log_msgs: &mut Vec<String>,
) -> Result<bool, String> {
    log_msgs.push(format!("Reading calibrations from binary."));
    for cal in &mut *calibrations {
        cal.value_repr = None;
        if cal.address.is_some() && cal.dtype.is_some() && cal.size.is_some() && cal.dim.is_some() {
            let a = cal.address.unwrap() as usize;
            let s = cal.size.unwrap() as usize;
            let d = cal.dim.unwrap() as usize;
            let range = a..a + (s * d);
            let val = binfile.get_values_by_address_range(range);
            if let Some(val_vec) = val {
                match datatype::bytes_to_text(
                    &val_vec,
                    cal.dtype.as_ref().unwrap(),
                    d,
                    &cal.endianess,
                ) {
                    Ok(x) => {
                        log_msgs.push(format!("CAL: {}: {}", &cal.symbol, &x));
                        cal.value_repr = Some(x)
                    }
                    Err(e) => log_msgs.push(format!("ERROR decoding {}: {}", &cal.symbol, &e)),
                }
            } else {
                log_msgs.push(format!("ERROR reading {}", &cal.symbol));
            }
        }
    }

    Ok(true)
}

fn write_calibration(
    calibrations: &Vec<Calibration>,
    binfile: &mut BinFile,
    log_msgs: &mut Vec<String>,
) -> Result<bool, String> {
    log_msgs.push(format!("Writing calibrations to binary."));
    for cal in calibrations {
        if cal.address.is_some()
            && cal.dtype.is_some()
            && cal.size.is_some()
            && cal.dim.is_some()
            && cal.value_repr.is_some()
        {
            let a = cal.address.unwrap() as usize;
            let d = cal.dim.unwrap() as usize;
            match datatype::text_to_bytes(
                &cal.value_repr.as_ref().unwrap(),
                cal.dtype.as_ref().unwrap(),
                d,
                &cal.endianess,
            ) {
                Ok(val) => {
                    log_msgs.push(format!(
                        "CAL: {}: {}",
                        &cal.symbol,
                        &cal.value_repr.as_ref().unwrap()
                    ));
                    let _ = binfile.add_bytes(val, Some(a), true);
                }
                Err(e) => log_msgs.push(format!("ERROR encoding {}: {}", &cal.symbol, &e)),
            }
        } else {
            log_msgs.push(format!("ERROR writing {}", &cal.symbol));
        }
    }

    Ok(true)
}

fn guess_binfile_format(
    binary_file: &OsString,
) -> (
    Option<BinFileFormat>,
    Option<SRecordAddressLength>,
    Option<IHexFormat>,
) {
    let mut binfile_format: Option<BinFileFormat> = None;
    let mut srec_addr_len: Option<SRecordAddressLength> = None;
    let mut ihex_format: Option<IHexFormat> = None;

    if let Some(ext) = Path::new(binary_file)
        .extension()
        .and_then(|ext| ext.to_str())
    {
        let ext_lower = ext.to_lowercase();

        match ext_lower.as_str() {
            "srec" | "s19" | "s28" | "s37" => {
                binfile_format = Some(BinFileFormat::SREC);
                srec_addr_len = Some(SRecordAddressLength::Length32);
            }
            "hex" | "ihex" => {
                binfile_format = Some(BinFileFormat::IHEX);
                ihex_format = Some(IHexFormat::IHex32);
            }
            _ => {}
        };
    }

    (binfile_format, srec_addr_len, ihex_format)
}

fn save_binfile(
    binary_file: &OsString,
    binfile: &BinFile,
    _log_msgs: &mut Vec<String>,
) -> Result<bool, String> {
    let (binfile_format, srec_addr_len, ihex_format) = guess_binfile_format(binary_file);

    let text: Vec<String> = match binfile_format {
        Some(BinFileFormat::SREC) => binfile
            .to_srec(
                None,
                srec_addr_len.unwrap_or(SRecordAddressLength::Length32),
            )
            .unwrap(),
        Some(BinFileFormat::IHEX) => binfile
            .to_ihex(None, ihex_format.unwrap_or(IHexFormat::IHex32))
            .unwrap(),
        _ => {
            return Err(String::from("Unrecognized binary file format"));
        }
    };

    let mut file = File::create(binary_file).expect("Error opening binary file for write");
    for line in text {
        writeln!(file, "{}", line).expect("Error writing binary file");
    }

    Ok(true)
}
