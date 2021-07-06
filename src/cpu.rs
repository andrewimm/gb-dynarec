
#[repr(C, packed)]
pub struct Registers {
  pub af: u32,
  pub bc: u32,
  pub de: u32,
  pub hl: u32,
  pub sp: u32,
  pub ip: u32,
}

impl Registers {
  pub fn new() -> Self {
    Self {
      af: 0,
      bc: 0,
      de: 0,
      hl: 0,
      sp: 0,
      ip: 0,
    }
  }

  pub fn after_boot() -> Self {
    Self {
      af: 0x01b0,
      bc: 0x0013,
      de: 0x00d8,
      hl: 0x014d,
      sp: 0xfffe,
      ip: 0x0100,
    }
  }

  pub fn get_af(&self) -> u32 {
    self.af
  }

  pub fn get_bc(&self) -> u32 {
    self.bc
  }

  pub fn get_de(&self) -> u32 {
    self.de
  }

  pub fn get_hl(&self) -> u32 {
    self.hl
  }

  pub fn get_sp(&self) -> u32 {
    self.sp
  }

  pub fn get_ip(&self) -> u32 {
    self.ip
  }
}

impl core::fmt::Debug for Registers {
  fn fmt(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
    let af = self.af;
    let bc = self.bc;
    let de = self.de;
    let hl = self.hl;
    let sp = self.sp;
    let ip = self.ip;
    formatter.debug_struct("Register")
      .field("AF", &format_args!("{:#06x}", af))
      .field("BC", &format_args!("{:#06x}", bc))
      .field("DE", &format_args!("{:#06x}", de))
      .field("HL", &format_args!("{:#06x}", hl))
      .field("SP", &format_args!("{:#06x}", sp))
      .field("IP", &format_args!("{:#06x}", ip))
      .finish()
  }
}

pub const STATUS_NORMAL: u8 = 0;
pub const STATUS_STOP: u8 = 1;
pub const STATUS_HALT: u8 = 2;
pub const STATUS_INTERRUPT_DISABLE: u8 = 3;
pub const STATUS_INTERRUPT_ENABLE: u8 = 4;
