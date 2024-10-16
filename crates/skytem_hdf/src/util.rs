use hdf5::{
    types::{IntSize, TypeDescriptor, VarLenAscii, VarLenUnicode},
    Attribute,
};

/// Reads an HDF5 attribute's value as a HDF5 string type and returns it as a native [`String`].
///
/// If the value is not a string type, an error is returned.
pub fn read_string_attribute(attr: &Attribute) -> hdf5::Result<String> {
    // Get the data type descriptor for the attribute
    match attr.dtype()?.to_descriptor()? {
        // Handle variable-length ASCII string
        TypeDescriptor::VarLenAscii => {
            let value: VarLenAscii = attr.read_scalar()?;
            Ok(value.as_str().to_owned())
        }
        // Handle variable-length UTF-8 string
        TypeDescriptor::VarLenUnicode => {
            let value: VarLenUnicode = attr.read_scalar()?;
            Ok(value.as_str().to_owned())
        }
        // Handle fixed-length ASCII string
        TypeDescriptor::FixedAscii(_) => {
            let buf = attr.read_raw()?;
            let string = String::from_utf8_lossy(&buf)
                .trim_end_matches('\0')
                .to_owned();
            Ok(string)
        }
        // Handle fixed-length UTF-8 string
        TypeDescriptor::FixedUnicode(_) => {
            let buf = attr.read_raw()?;
            let string = String::from_utf8_lossy(&buf)
                .trim_end_matches('\0')
                .to_owned();
            Ok(string)
        }
        // Unsupported data type
        _ => Err(hdf5::Error::from("Unsupported attribute type")),
    }
}

/// Reads an HDF5 attribute's value and converts it to a native [`String`].
pub fn read_any_attribute_to_string(attr: &Attribute) -> hdf5::Result<String> {
    // Get the data type descriptor for the attribute
    let type_descriptor = attr.dtype()?.to_descriptor()?;
    match &type_descriptor {
        // Handle variable-length ASCII string
        TypeDescriptor::VarLenAscii => {
            let value: VarLenAscii = attr.read_scalar()?;
            Ok(value.as_str().to_owned())
        }
        // Handle variable-length UTF-8 string
        TypeDescriptor::VarLenUnicode => {
            let value: VarLenUnicode = attr.read_scalar()?;
            Ok(value.as_str().to_owned())
        }
        // Handle fixed-length ASCII string
        TypeDescriptor::FixedAscii(_) => {
            let buf = attr.read_raw()?;
            let string = String::from_utf8_lossy(&buf)
                .trim_end_matches('\0')
                .to_owned();
            Ok(string)
        }
        // Handle fixed-length UTF-8 string
        TypeDescriptor::FixedUnicode(_) => {
            let buf = attr.read_raw()?;
            let string = String::from_utf8_lossy(&buf)
                .trim_end_matches('\0')
                .to_owned();
            Ok(string)
        }
        TypeDescriptor::Integer(int_size) => {
            let value: String = match int_size {
                IntSize::U1 => attr.read_scalar::<i8>()?.to_string(),
                IntSize::U2 => attr.read_scalar::<i16>()?.to_string(),
                IntSize::U4 => attr.read_scalar::<i32>()?.to_string(),
                IntSize::U8 => attr.read_scalar::<i64>()?.to_string(),
            };
            Ok(value)
        }
        TypeDescriptor::Unsigned(int_size) => {
            let value: String = match int_size {
                hdf5::types::IntSize::U1 => attr.read_scalar::<u8>()?.to_string(),
                hdf5::types::IntSize::U2 => attr.read_scalar::<u16>()?.to_string(),
                hdf5::types::IntSize::U4 => attr.read_scalar::<u32>()?.to_string(),
                hdf5::types::IntSize::U8 => attr.read_scalar::<u64>()?.to_string(),
            };
            Ok(value)
        }
        TypeDescriptor::Float(float_size) => {
            let value: String = match float_size {
                hdf5::types::FloatSize::U4 => attr.read_scalar::<f32>()?.to_string(),
                hdf5::types::FloatSize::U8 => attr.read_scalar::<f64>()?.to_string(),
            };
            Ok(value)
        }
        TypeDescriptor::Boolean => {
            let value: bool = attr.read_scalar()?;
            Ok(value.to_string())
        }
        TypeDescriptor::Enum(enum_type) => {
            let value: u64 = attr.read_scalar()?;
            let enum_name = enum_type.members.get(value as usize).map_or_else(
                || format!("Unknown Enum: {type_descriptor}"),
                |member| member.name.clone(),
            );
            Ok(enum_name)
        }
        _ => Err(hdf5::Error::from(format!(
            "Unsupported attribute type: {type_descriptor}"
        ))),
    }
}
