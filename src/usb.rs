use std::mem;
use libusb1_sys::constants as libusb;

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
struct InterfaceDesc {
    length: u8,
    desc_type: u8,
    interface_num: u8,
    alt_setting: u8,
    num_endpoints: u8,
    class: u8,
    subclass: u8,
    proto: u8,
    interface: u8,
}

const USB_DT_INTERFACE_SIZE: usize = 9;

const FASTBOOT_INTERFACE: InterfaceDesc = InterfaceDesc {
    length: USB_DT_INTERFACE_SIZE as u8,
    desc_type: libusb::LIBUSB_DT_INTERFACE,
    interface_num: 0,
    alt_setting: 0,
    num_endpoints: 2,
    class: libusb::LIBUSB_CLASS_VENDOR_SPEC,
    subclass: 66,
    proto: 3,
    interface: 1,
};

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
struct EndpointDescNoAudio {
    length: u8,
    desc_type: u8,
    addr: u8,
    attr: u8,
    max_packet_size: u16,
    interval: u8,
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
struct FuncDesc {
    interface: InterfaceDesc,
    source: EndpointDescNoAudio,
    sink: EndpointDescNoAudio,
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
struct SsEpCompDesc {
    length: u8,
    desc_type: u8,
    max_burst: u8,
    attr: u8,
    bytes_per_interval: u16,
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
struct SsFuncDesc {
    interface: InterfaceDesc,
    source: EndpointDescNoAudio,
    source_comp: SsEpCompDesc,
    sink: EndpointDescNoAudio,
    sink_comp: SsEpCompDesc,
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct DescV2 {
    magic: u32,
    length: u32,
    flags: u32,
    fs_count: u32,
    hs_count: u32,
    ss_count: u32,
    fs_descs: FuncDesc,
    hs_descs: FuncDesc,
    ss_descs: SsFuncDesc,
}

impl DescV2 {
    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                self as *const DescV2 as *const u8,
                mem::size_of::<DescV2>(),
            )
        }
    }
}

const FUNCTIONFS_STRINGS_MAGIC: u32 = 2_u32.to_le();
const FUNCTIONFS_DESCRIPTORS_MAGIC_V2: u32 = 3_u32.to_le();

const FFS_HAS_FS_DESC: u32 = 1_u32.to_le();
const FFS_HAS_HS_DESC: u32 = 2_u32.to_le();
const FFS_HAS_SS_DESC: u32 = 4_u32.to_le();

const MAX_PACKET_SIZE_FS: u16 = 64_u16.to_le();
const MAX_PACKET_SIZE_HS: u16 = 512_u16.to_le();
const MAX_PACKET_SIZE_SS: u16 = 1024_u16.to_le();

const USB_DIR_OUT: u8 = 0;
const USB_DIR_IN: u8 = 0x80;

const USB_ENDPOINT_XFER_BULK: u8 = 2;

pub const FASTBOOT_DESCRIPTOR_V2: DescV2 = DescV2 {
    magic: FUNCTIONFS_DESCRIPTORS_MAGIC_V2,
    length: (mem::size_of::<DescV2>() as u32).to_le(),
    flags: FFS_HAS_FS_DESC | FFS_HAS_HS_DESC | FFS_HAS_SS_DESC,
    fs_count: 3_u32.to_le(),
    hs_count: 3_u32.to_le(),
    ss_count: 5_u32.to_le(),
    fs_descs: FASTBOOT_FS_DESC,
    hs_descs: FASTBOOT_HS_DESC,
    ss_descs: FASTBOOT_SS_DESC,
};

const FASTBOOT_FS_DESC: FuncDesc = FuncDesc {
    interface: FASTBOOT_INTERFACE,
    source: EndpointDescNoAudio {
        length: mem::size_of::<EndpointDescNoAudio>() as u8,
        desc_type: libusb::LIBUSB_DT_ENDPOINT,
        addr: 1 | USB_DIR_OUT,
        attr: USB_ENDPOINT_XFER_BULK,
        max_packet_size: MAX_PACKET_SIZE_FS,
        interval: 0,
    },
    sink: EndpointDescNoAudio {
        length: mem::size_of::<EndpointDescNoAudio>() as u8,
        desc_type: libusb::LIBUSB_DT_ENDPOINT,
        addr: 1 | USB_DIR_IN,
        attr: USB_ENDPOINT_XFER_BULK,
        max_packet_size: MAX_PACKET_SIZE_FS,
        interval: 0,
    },
};

const FASTBOOT_HS_DESC: FuncDesc = FuncDesc {
    interface: FASTBOOT_INTERFACE,
    source: EndpointDescNoAudio {
        length: mem::size_of::<EndpointDescNoAudio>() as u8,
        desc_type: libusb::LIBUSB_DT_ENDPOINT,
        addr: 1 | USB_DIR_OUT,
        attr: USB_ENDPOINT_XFER_BULK,
        max_packet_size: MAX_PACKET_SIZE_HS,
        interval: 0,
    },
    sink: EndpointDescNoAudio {
        length: mem::size_of::<EndpointDescNoAudio>() as u8,
        desc_type: libusb::LIBUSB_DT_ENDPOINT,
        addr: 1 | USB_DIR_IN,
        attr: USB_ENDPOINT_XFER_BULK,
        max_packet_size: MAX_PACKET_SIZE_HS,
        interval: 0,
    },
};

const FASTBOOT_SS_DESC: SsFuncDesc = SsFuncDesc {
    interface: FASTBOOT_INTERFACE,
    source: EndpointDescNoAudio {
        length: mem::size_of::<EndpointDescNoAudio>() as u8,
        desc_type: libusb::LIBUSB_DT_ENDPOINT,
        addr: 1 | USB_DIR_OUT,
        attr: USB_ENDPOINT_XFER_BULK,
        max_packet_size: MAX_PACKET_SIZE_SS,
        interval: 0,
    },
    source_comp: SsEpCompDesc {
        length: mem::size_of::<SsEpCompDesc>() as u8,
        desc_type: libusb::LIBUSB_DT_SS_ENDPOINT_COMPANION,
        max_burst: 15,
        attr: 0,
        bytes_per_interval: 0,
    },
    sink: EndpointDescNoAudio {
        length: mem::size_of::<EndpointDescNoAudio>() as u8,
        desc_type: libusb::LIBUSB_DT_ENDPOINT,
        addr: 1 | USB_DIR_IN,
        attr: USB_ENDPOINT_XFER_BULK,
        max_packet_size: MAX_PACKET_SIZE_SS,
        interval: 0,
    },
    sink_comp: SsEpCompDesc {
        length: mem::size_of::<SsEpCompDesc>() as u8,
        desc_type: libusb::LIBUSB_DT_SS_ENDPOINT_COMPANION,
        max_burst: 15,
        attr: 0,
        bytes_per_interval: 0,
    },
};

const IFACE_STRING: &'static [u8; 10] = b"fastbootd\0";

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct FfsStringData {
    magic: u32,
    length: u32,
    str_count: u32,
    lang_count: u32,
    code: u16,
    str1: [u8; IFACE_STRING.len()],
}

impl FfsStringData {
    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                self as *const FfsStringData as *const u8,
                mem::size_of::<FfsStringData>(),
            )
        }
    }
}

pub const FASTBOOT_STRINGS: FfsStringData = FfsStringData {
    magic: FUNCTIONFS_STRINGS_MAGIC,
    length: (mem::size_of::<FfsStringData>() as u32).to_le(),
    str_count: 1,
    lang_count: 1,

    code: 0x409_u16.to_le(),
    str1: *IFACE_STRING,
};
