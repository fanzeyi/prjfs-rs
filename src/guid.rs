use winapi::shared::guiddef::GUID;
use winapi::um::combaseapi::CoCreateGuid;

pub fn create_guid() -> GUID {
    let mut guid: GUID = Default::default();
    unsafe { CoCreateGuid(&mut guid) };
    guid
}

pub fn guid_from_bytes(bytes: Vec<u8>) -> std::result::Result<GUID, std::option::NoneError> {
    let mut guid = bytes.into_iter();
    Ok(GUID {
        Data1: u32::from_be_bytes([guid.next()?, guid.next()?, guid.next()?, guid.next()?]),
        Data2: u16::from_be_bytes([guid.next()?, guid.next()?]),
        Data3: u16::from_be_bytes([guid.next()?, guid.next()?]),
        Data4: [
            guid.next()?,
            guid.next()?,
            guid.next()?,
            guid.next()?,
            guid.next()?,
            guid.next()?,
            guid.next()?,
            guid.next()?,
        ],
    })
}

pub fn guid_to_bytes(guid: &GUID) -> Vec<u8> {
    [
        &guid.Data1.to_be_bytes()[..],
        &guid.Data2.to_be_bytes(),
        &guid.Data3.to_be_bytes(),
        &guid.Data4,
    ]
    .concat()
}
