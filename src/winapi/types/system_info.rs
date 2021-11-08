use super::*;


#[derive(Clone, Copy)]
#[repr(C)]
pub struct SYSTEM_INFO_U_DUMMY {
    pub wProcessorArchitecture: WORD,
    pub wReserved: WORD,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub union SYSTEM_INFO_U {
    pub dwOemId: DWORD,
    pub DUMMYSTRUCTNAME: SYSTEM_INFO_U_DUMMY,
}
