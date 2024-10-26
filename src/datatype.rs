use crate::dwarf::{DwarfDataType, TypeInfo};
use a2lfile::DataType;

// map the datatypes from the elf_info to a2l datatypes
// the only really relevant cases are for the integer, floating point and enum types
// all other types cannot be sensibly measured / calibrated anyway
pub(crate) fn get_a2l_datatype(typeinfo: &TypeInfo) -> DataType {
    match &typeinfo.datatype {
        DwarfDataType::Uint8 => DataType::Ubyte,
        DwarfDataType::Uint16 => DataType::Uword,
        DwarfDataType::Uint32 => DataType::Ulong,
        DwarfDataType::Uint64 => DataType::AUint64,
        DwarfDataType::Sint8 => DataType::Sbyte,
        DwarfDataType::Sint16 => DataType::Sword,
        DwarfDataType::Sint32 => DataType::Slong,
        DwarfDataType::Sint64 => DataType::AInt64,
        DwarfDataType::Float => DataType::Float32Ieee,
        DwarfDataType::Double => DataType::Float64Ieee,
        DwarfDataType::Bitfield { basetype, .. } => get_a2l_datatype(basetype),
        DwarfDataType::Pointer(size, _) => {
            if *size == 8 {
                DataType::AUint64
            } else {
                DataType::Ulong
            }
        }
        DwarfDataType::Enum { size, .. } | DwarfDataType::Other(size) => match *size {
            8 => DataType::AUint64,
            4 => DataType::Ulong,
            2 => DataType::Uword,
            _ => DataType::Ubyte,
        },
        DwarfDataType::Array { arraytype, .. } => get_a2l_datatype(arraytype),
        _ => DataType::Ubyte,
    }
}

pub(crate) fn get_type_limits(
    typeinfo: &TypeInfo,
    default_lower: f64,
    default_upper: f64,
) -> (f64, f64) {
    let (new_lower_limit, new_upper_limit) = match &typeinfo.datatype {
        DwarfDataType::Array { arraytype, .. } => {
            get_type_limits(arraytype, default_lower, default_upper)
        }
        DwarfDataType::Bitfield {
            bit_size, basetype, ..
        } => {
            let raw_range: u64 = 1 << bit_size;
            match &basetype.datatype {
                DwarfDataType::Sint8
                | DwarfDataType::Sint16
                | DwarfDataType::Sint32
                | DwarfDataType::Sint64 => {
                    let lower = -((raw_range / 2) as f64);
                    let upper = (raw_range / 2) as f64;
                    (lower, upper)
                }
                _ => (0f64, raw_range as f64),
            }
        }
        DwarfDataType::Double => (f64::MIN, f64::MAX),
        DwarfDataType::Float => (f64::from(f32::MIN), f64::from(f32::MAX)),
        DwarfDataType::Uint8 => (f64::from(u8::MIN), f64::from(u8::MAX)),
        DwarfDataType::Uint16 => (f64::from(u16::MIN), f64::from(u16::MAX)),
        DwarfDataType::Uint32 => (f64::from(u32::MIN), f64::from(u32::MAX)),
        DwarfDataType::Uint64 => (u64::MIN as f64, u64::MAX as f64),
        DwarfDataType::Sint8 => (f64::from(i8::MIN), f64::from(i8::MAX)),
        DwarfDataType::Sint16 => (f64::from(i16::MIN), f64::from(i16::MAX)),
        DwarfDataType::Sint32 => (f64::from(i32::MIN), f64::from(i32::MAX)),
        DwarfDataType::Sint64 => (i64::MIN as f64, i64::MAX as f64),
        DwarfDataType::Enum { enumerators, .. } => {
            let lower = enumerators.iter().map(|val| val.1).min().unwrap_or(0) as f64;
            let upper = enumerators.iter().map(|val| val.1).max().unwrap_or(0) as f64;
            (lower, upper)
        }
        _ => (default_lower, default_upper),
    };
    (new_lower_limit, new_upper_limit)
}

pub(crate) fn get_datatype_size(datatype: &DataType) -> u16 {
    match datatype {
        DataType::Ubyte => 1,
        DataType::Sbyte => 1,
        DataType::Uword => 2,
        DataType::Sword => 2,
        DataType::Ulong => 4,
        DataType::Slong => 4,
        DataType::AUint64 => 8,
        DataType::AInt64 => 8,
        DataType::Float16Ieee => 2,
        DataType::Float32Ieee => 4,
        DataType::Float64Ieee => 8,
    }
}

pub(crate) fn bytes_to_text(bytes: &[u8], datatype: &DataType, dim: usize) -> Result<String, &'static str> {
    let size = get_datatype_size(datatype) as usize;
    if bytes.len() != dim * size {
        Err("Size mismatch")
    } else {
        if dim == 1 {
            match datatype {
                DataType::Ubyte => Ok(bytes[0].to_string()),
                DataType::Sbyte => {
                    let x = bytes[0] as i8;
                    Ok(x.to_string())
                },
                DataType::Uword => {
                    let x = u16::from_le_bytes(bytes[0..size].try_into().unwrap());
                    Ok(x.to_string())
                },
                DataType::Sword => {
                    let x = i16::from_le_bytes(bytes[0..size].try_into().unwrap());
                    Ok(x.to_string())
                },
                DataType::Ulong => {
                    let x = u32::from_le_bytes(bytes[0..size].try_into().unwrap());
                    Ok(x.to_string())
                },
                DataType::Slong => {
                    let x = i32::from_le_bytes(bytes[0..size].try_into().unwrap());
                    Ok(x.to_string())
                },
                DataType::AUint64 => {
                    let x = u64::from_le_bytes(bytes[0..size].try_into().unwrap());
                    Ok(x.to_string())
                },
                DataType::AInt64 => {
                    let x = i64::from_le_bytes(bytes[0..size].try_into().unwrap());
                    Ok(x.to_string())
                },
                DataType::Float16Ieee => {
                    // let x = f16::from_le_bytes(bytes[0..size].try_into().unwrap());
                    // Ok(x.to_string())
                    Err("Float16Ieee is not supported")
                },
                DataType::Float32Ieee => {
                    let x = f32::from_le_bytes(bytes[0..size].try_into().unwrap());
                    Ok(x.to_string())
                },
                DataType::Float64Ieee => {
                    let x = f64::from_le_bytes(bytes[0..size].try_into().unwrap());
                    Ok(x.to_string())
                },
            }
        } else if dim > 1 {
            let mut repr = String::from("(");
            let mut sep = "";
            for i in 0..dim {
                repr.push_str(&sep);
                repr.push_str(& bytes_to_text(&bytes[i*size..(i+1)*size], datatype, 1)?);
                sep = ",";
            }
            repr.push(')');
            Ok(repr)
        } else {
            Err("Dimension zero is not allowed")
        }
    }
}

pub(crate) fn text_to_bytes(text: &str, datatype: &DataType, dim: usize) -> Result<Vec<u8>, &'static str> {
    let text = text.trim();
    if text.starts_with('(') && text.ends_with(')') {
        let numbers_str = &text[1..text.len()-1];

        let numbers: Vec<&str> = numbers_str
            .split(',')
            .map(|num_str| num_str.trim())
            .collect();

        if numbers.len() == dim {
            let size = get_datatype_size(datatype) as usize;
            let mut ret = Vec::with_capacity(dim * size);
            for i in 0..dim {
                ret.append(&mut text_to_bytes(&numbers[i], datatype, 1)?);
            }
            Ok(ret)
        } else {
            Err("Dimensions mismatch")
        }
    } else if text.starts_with('"') && text.ends_with('"') {
        let bytes = text[1..text.len()-1].as_bytes();
        let size = get_datatype_size(datatype) as usize;
        if bytes.len() <= dim * size {
            let mut ret = bytes.to_vec();
            ret.resize(dim * size, 0u8);
            Ok(ret)
        } else {
            Err("Dimensions mismatch")
        }
    } else {
        match datatype {
            DataType::Ubyte => {
                let n = text.parse::<u8>();
                if n.is_ok() {
                    Ok(Vec::from(n.unwrap().to_le_bytes()))
                } else {
                    Err("Error parsing number")
                }
            },
            DataType::Sbyte => {
                let n = text.parse::<i8>();
                if n.is_ok() {
                    Ok(Vec::from(n.unwrap().to_le_bytes()))
                } else {
                    Err("Error parsing number")
                }
            },
            DataType::Uword => {
                let n = text.parse::<u16>();
                if n.is_ok() {
                    Ok(Vec::from(n.unwrap().to_le_bytes()))
                } else {
                    Err("Error parsing number")
                }
            },
            DataType::Sword => {
                let n = text.parse::<i16>();
                if n.is_ok() {
                    Ok(Vec::from(n.unwrap().to_le_bytes()))
                } else {
                    Err("Error parsing number")
                }
            },
            DataType::Ulong => {
                let n = text.parse::<u32>();
                if n.is_ok() {
                    Ok(Vec::from(n.unwrap().to_le_bytes()))
                } else {
                    Err("Error parsing number")
                }
            },
            DataType::Slong => {
                let n = text.parse::<i32>();
                if n.is_ok() {
                    Ok(Vec::from(n.unwrap().to_le_bytes()))
                } else {
                    Err("Error parsing number")
                }
            },
            DataType::AUint64 => {
                let n = text.parse::<u64>();
                if n.is_ok() {
                    Ok(Vec::from(n.unwrap().to_le_bytes()))
                } else {
                    Err("Error parsing number")
                }
            },
            DataType::AInt64 => {
                let n = text.parse::<i64>();
                if n.is_ok() {
                    Ok(Vec::from(n.unwrap().to_le_bytes()))
                } else {
                    Err("Error parsing number")
                }
            },
            DataType::Float16Ieee => {
                Err("Float16Ieee not supported")
            },
            DataType::Float32Ieee => {
                let n = text.parse::<f32>();
                if n.is_ok() {
                    Ok(Vec::from(n.unwrap().to_le_bytes()))
                } else {
                    Err("Error parsing number")
                }
            },
            DataType::Float64Ieee => {
                let n = text.parse::<f64>();
                if n.is_ok() {
                    Ok(Vec::from(n.unwrap().to_le_bytes()))
                } else {
                    Err("Error parsing number")
                }
            },
        }
    }
}
