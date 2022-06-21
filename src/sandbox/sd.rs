use crate::device::raspi3b::base::*;
use crate::*;
use bitflags::bitflags;
use core::arch::asm;

// https://github.com/bztsrc/raspi3-tutorial/blob/master/0B_readsector/sd.c

// #define EMMC_ARG2           ((volatile unsigned int*)(MMIO_BASE+0x00300000))
pub const EMMC_ARG2: usize = MMIO_BASE + 0x300000;
// #define EMMC_BLKSIZECNT     ((volatile unsigned int*)(MMIO_BASE+0x00300004))
pub const EMMC_BLKSIZECNT: usize = MMIO_BASE + 0x300004;
// #define EMMC_ARG1           ((volatile unsigned int*)(MMIO_BASE+0x00300008))
pub const EMMC_ARG1: usize = MMIO_BASE + 0x300008;
// #define EMMC_CMDTM          ((volatile unsigned int*)(MMIO_BASE+0x0030000C))
pub const EMMC_CMDTM: usize = MMIO_BASE + 0x30000C;
// #define EMMC_RESP0          ((volatile unsigned int*)(MMIO_BASE+0x00300010))
pub const EMMC_RESP0: usize = MMIO_BASE + 0x300010;
// #define EMMC_RESP1          ((volatile unsigned int*)(MMIO_BASE+0x00300014))
pub const EMMC_RESP1: usize = MMIO_BASE + 0x300014;
// #define EMMC_RESP2          ((volatile unsigned int*)(MMIO_BASE+0x00300018))
pub const EMMC_RESP2: usize = MMIO_BASE + 0x300018;
// #define EMMC_RESP3          ((volatile unsigned int*)(MMIO_BASE+0x0030001C))
pub const EMMC_RESP3: usize = MMIO_BASE + 0x30001C;
// #define EMMC_DATA           ((volatile unsigned int*)(MMIO_BASE+0x00300020))
pub const EMMC_DATA: usize = MMIO_BASE + 0x300020;
// #define EMMC_STATUS         ((volatile unsigned int*)(MMIO_BASE+0x00300024))
pub const EMMC_STATUS: usize = MMIO_BASE + 0x300024;
// #define EMMC_CONTROL0       ((volatile unsigned int*)(MMIO_BASE+0x00300028))
pub const EMMC_CONTROL0: usize = MMIO_BASE + 0x300028;
// #define EMMC_CONTROL1       ((volatile unsigned int*)(MMIO_BASE+0x0030002C))
pub const EMMC_CONTROL1: usize = MMIO_BASE + 0x30002C;
// #define EMMC_INTERRUPT      ((volatile unsigned int*)(MMIO_BASE+0x00300030))
pub const EMMC_INTERRUPT: usize = MMIO_BASE + 0x300030;
// #define EMMC_INT_MASK       ((volatile unsigned int*)(MMIO_BASE+0x00300034))
pub const EMMC_INT_MASK: usize = MMIO_BASE + 0x300034;
// #define EMMC_INT_EN         ((volatile unsigned int*)(MMIO_BASE+0x00300038))
pub const EMMC_INT_EN: usize = MMIO_BASE + 0x300038;
// #define EMMC_CONTROL2       ((volatile unsigned int*)(MMIO_BASE+0x0030003C))
pub const EMMC_CONTROL2: usize = MMIO_BASE + 0x30003C;
// #define EMMC_SLOTISR_VER    ((volatile unsigned int*)(MMIO_BASE+0x003000FC))
pub const EMMC_SLOTISR_VER: usize = MMIO_BASE + 0x3000FC;

bitflags! {
    struct Command: u32 {
// #define CMD_NEED_APP        0x80000000
        const CMD_NEED_APP = 0x80000000;
// #define CMD_RSPNS_48        0x00020000
        const CMD_RSPNS_48 = 0x00020000;
// #define CMD_ERRORS_MASK     0xfff9c004
        const CMD_ERRORS_MASK = 0xfff9c004;
// #define CMD_RCA_MASK        0xffff0000
        const CMD_RCA_MASK = 0xffff0000;
// #define CMD_GO_IDLE         0x00000000
        const CMD_GO_IDLE = 0x00000000;
// #define CMD_ALL_SEND_CID    0x02010000
        const CMD_ALL_SEND_CID = 0x02010000;
// #define CMD_SEND_REL_ADDR   0x03020000
        const CMD_SEND_REL_ADDR = 0x03020000;
// #define CMD_CARD_SELECT     0x07030000
        const CMD_CARD_SELECT = 0x07030000;
// #define CMD_SEND_IF_COND    0x08020000
        const CMD_SEND_IF_COND = 0x08020000;
// #define CMD_STOP_TRANS      0x0C030000
        const CMD_SEND_STOP_TRANS = 0x0C030000;
// #define CMD_READ_SINGLE     0x11220010
        const CMD_READ_SINGLE = 0x11220010;
// #define CMD_READ_MULTI      0x12220032
        const CMD_READ_MULTI = 0x12220032;
// #define CMD_SET_BLOCKCNT    0x17020000
        const CMD_SET_BLOCKCNT = 0x17020000;
// #define CMD_APP_CMD         0x37000000
        const CMD_APP_CMD = 0x37000000;
// #define CMD_SET_BUS_WIDTH   (0x06020000|CMD_NEED_APP)
        const CMD_SET_BUS_WIDTH = 0x06020000 | Self::CMD_NEED_APP.bits;
// #define CMD_SEND_OP_COND    (0x29020000|CMD_NEED_APP)
        const CMD_SEND_OP_COND = 0x29020000 | Self::CMD_NEED_APP.bits;
// #define CMD_SEND_SCR        (0x33220010|CMD_NEED_APP)
        const CMD_SEND_SCR = 0x33220010 | Self::CMD_NEED_APP.bits;
    }

    struct Status: u32 {
// #define SR_READ_AVAILABLE   0x00000800
        const SR_READ_AVAILABLE = 0x00000800;
// #define SR_DAT_INHIBIT      0x00000002
        const SR_DAT_INHIBIT = 0x00000002;
// #define SR_CMD_INHIBIT      0x00000001
        const SR_CMD_INHIBIT = 0x00000001;
// #define SR_APP_CMD          0x00000020
        const SR_APP_CMD = 0x00000020;
    }

    struct Interrupt: u32 {
// #define INT_DATA_TIMEOUT    0x00100000
        const INT_DATA_TIMEOUT = 0x00100000;
// #define INT_CMD_TIMEOUT     0x00010000
        const INT_CMD_TIMEOUT = 0x00010000;
// #define INT_READ_RDY        0x00000020
        const INT_READ_RDY = 0x00000020;
// #define INT_CMD_DONE        0x00000001
        const INT_CMD_DONE = 0x00000001;
// #define INT_ERROR_MASK      0x017E8000
        const INT_ERROR_MASK = 0x017E8000;
    }

    struct Control: u32 {
// #define C0_SPI_MODE_EN      0x00100000
        const C0_SPI_MODE_EN = 0x00100000;
// #define C0_HCTL_HS_EN       0x00000004
        const C0_HCTL_HS_EN = 0x00000004;
// #define C0_HCTL_DWITDH      0x00000002
        const C0_HCTL_DWITDH = 0x00000002;
//
// #define C1_SRST_DATA        0x04000000
        const C1_SRST_DATA = 0x04000000;
// #define C1_SRST_CMD         0x02000000
        const C1_SRST_CMD = 0x02000000;
// #define C1_SRST_HC          0x01000000
        const C1_SRST_HC = 0x01000000;
// #define C1_TOUNIT_DIS       0x000f0000
        const C1_TOUNIT_DIS = 0x000f0000;
// #define C1_TOUNIT_MAX       0x000e0000
        const C1_TOUNIT_MAX = 0x000e0000;
// #define C1_CLK_GENSEL       0x00000020
        const C1_CLK_GENSEL = 0x00000020;
// #define C1_CLK_EN           0x00000004
        const C1_CLK_EN = 0x00000004;
// #define C1_CLK_STABLE       0x00000002
        const C1_CLK_STABLE = 0x00000002;
// #define C1_CLK_INTLEN       0x00000001
        const C1_CLK_INTLEN = 0x00000001a;
    }

    struct SlotISRVer: u32 {
// #define HOST_SPEC_NUM       0x00ff0000
        const HOST_SPEC_NUM = 0x00ff0000;
// #define HOST_SPEC_NUM_SHIFT 16
        const HOST_SPEC_NUM_SHIFT = 16;
// #define HOST_SPEC_V3        2
        const HOST_SPEC_V3 = 2;
// #define HOST_SPEC_V2        1
        const HOST_SPEC_V2 = 1;
// #define HOST_SPEC_V1        0
        const HOST_SPEC_V1 = 0;
    }

    struct ScrFlag: u32 {
// #define SCR_SD_BUS_WIDTH_4  0x00000400
        const SCR_SD_BUS_WIDTH_4 = 0x00000400;
// #define SCR_SUPP_SET_BLKCNT 0x02000000
        const SCR_SUPP_SET_BLKCNT = 0x02000000;
// #define SCR_SUPP_CCS        0x00000001
        const SCR_SUPP_CCS = 0x00000001;
// #define ACMD41_VOLTAGE      0x00ff8000
        const ACMD41_VOLTAGE = 0x00ff8000;
// #define ACMD41_CMD_COMPLETE 0x80000000
        const ACMD41_CMD_COMPLETE = 0x80000000;
// #define ACMD41_CMD_CCS      0x40000000
        const ACMD41_CMD_CCS = 0x40000000;
// #define ACMD41_ARG_HC       0x51ff8000
        const ACMD41_ARG_HC = 0x51ff8000;
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum SDError {
    Error,
    TimeOut,
}

pub struct SDCard {
    sd_scr: [usize; 2],
    sd_ocr: usize,
    sd_rca: bool,
    sd_err: Option<SDError>,
    sd_hv: usize,
}

impl SDCard {
    fn read_reg(addr: usize) -> u32 {
        unsafe { (addr as *const u32).read_volatile() }
    }

    fn write_reg(addr: usize, data: u32) {
        unsafe { (addr as *mut u32).write_volatile(data) }
    }

    pub fn sd_status(&self, mask: u32) -> Result<(), SDError> {
        let mut count = 500000;
        while (Self::read_reg(EMMC_STATUS) & mask) != 0
            && Self::read_reg(EMMC_INTERRUPT) & Interrupt::INT_ERROR_MASK.bits() == 0
            && count != 0
        {
            count -= 1;
            wait_msec(1);
        }

        if count <= 0 || (Self::read_reg(EMMC_INTERRUPT) & Interrupt::INT_ERROR_MASK.bits()) != 0 {
            Err(SDError::Error)
        } else {
            Ok(())
        }
    }

    pub fn sd_int(&self, mask: u32) -> Result<(), SDError> {
        let (mut r, m) = (0, mask | Interrupt::INT_ERROR_MASK.bits());
        let mut count = 1000000;
        while Self::read_reg(EMMC_INTERRUPT) & m == 0 && count != 0 {
            count -= 1;
            wait_msec(1);
        }
        r = Self::read_reg(EMMC_INTERRUPT);
        if count <= 0
            || (r & Interrupt::INT_CMD_TIMEOUT.bits()) != 0
            || (r & Interrupt::INT_DATA_TIMEOUT.bits()) != 0
        {
            Self::write_reg(EMMC_INTERRUPT, r);
            return Err(SDError::TimeOut);
        }
        if (r & Interrupt::INT_ERROR_MASK.bits()) != 0 {
            Self::write_reg(EMMC_INTERRUPT, r);
            return Err(SDError::Error);
        }
        Self::write_reg(EMMC_INTERRUPT, mask);
        Ok(())
    }
}

pub fn wait_msec(n: u64) {
    let (mut f, mut t, mut r): (u64, u64, u64);
    unsafe {
        asm!("mrs {}, cntfrq_el0", out(reg)f);
        asm!("mrs {}, cntpct_el0", out(reg)t);
        t += ((f / 1000).wrapping_mul(n)) / 1000;
        asm!("mrs {}, cntpct_el0", out(reg)r);
        while r < t {
            asm!("mrs {}, cntpct_el0", out(reg)r);
        }
    }
}

pub fn test() {
    let mut n: u32 = 10;
    n |= Command::CMD_NEED_APP.bits();
    println!("{:#x}", n);
    for i in 0..10 {
        println!("{} sec", i);
        wait_msec(1000_000);
    }
}
