#[non_exhaustive]
pub struct AccountFlags;

#[allow(dead_code)]
impl AccountFlags {
    pub const ACCOUNT_FLAG_GM: u32      = 0x00000001;
    pub const ACCOUNT_FLAG_TRIAL: u32   = 0x00000008;
    pub const ACCOUNT_FLAG_PROPASS: u32 = 0x00800000;
}