#![allow(unused_assignments)]

use crate::device::raspi3b::base::*;
use crate::lazy::Lazy;
use bitflags::bitflags;
use core::arch::asm;
use log::{error, info};

// https://github.com/bztsrc/raspi3-tutorial/blob/master/0B_readsector/sd.c

pub static mut SDCARD: Lazy<SDCard> = Lazy::new(|| SDCard {
    sd_scr: [0; 2],
    sd_ocr: 0,
    sd_rca: 0,
    sd_err: 0,
    sd_hv: 0,
});

pub const EMMC_ARG2: usize = MMIO_BASE + 0x300000;
pub const EMMC_BLKSIZECNT: usize = MMIO_BASE + 0x300004;
pub const EMMC_ARG1: usize = MMIO_BASE + 0x300008;
pub const EMMC_CMDTM: usize = MMIO_BASE + 0x30000C;
pub const EMMC_RESP0: usize = MMIO_BASE + 0x300010;
pub const EMMC_RESP1: usize = MMIO_BASE + 0x300014;
pub const EMMC_RESP2: usize = MMIO_BASE + 0x300018;
pub const EMMC_RESP3: usize = MMIO_BASE + 0x30001C;
pub const EMMC_DATA: usize = MMIO_BASE + 0x300020;
pub const EMMC_STATUS: usize = MMIO_BASE + 0x300024;
pub const EMMC_CONTROL0: usize = MMIO_BASE + 0x300028;
pub const EMMC_CONTROL1: usize = MMIO_BASE + 0x30002C;
pub const EMMC_INTERRUPT: usize = MMIO_BASE + 0x300030;
pub const EMMC_INT_MASK: usize = MMIO_BASE + 0x300034;
pub const EMMC_INT_EN: usize = MMIO_BASE + 0x300038;
pub const EMMC_CONTROL2: usize = MMIO_BASE + 0x30003C;
pub const EMMC_SLOTISR_VER: usize = MMIO_BASE + 0x3000FC;

bitflags! {
    struct Command: u32 {
        const CMD_NEED_APP = 0x80000000;
        const CMD_RSPNS_48 = 0x00020000;
        const CMD_ERRORS_MASK = 0xfff9c004;
        const CMD_RCA_MASK = 0xffff0000;
        const CMD_GO_IDLE = 0x00000000;
        const CMD_ALL_SEND_CID = 0x02010000;
        const CMD_SEND_REL_ADDR = 0x03020000;
        const CMD_CARD_SELECT = 0x07030000;
        const CMD_SEND_IF_COND = 0x08020000;
        const CMD_STOP_TRANS = 0x0C030000;
        const CMD_READ_SINGLE = 0x11220010;
        const CMD_READ_MULTI = 0x12220032;
        const CMD_SET_BLOCKCNT = 0x17020000;
        const CMD_APP_CMD = 0x37000000;
        const CMD_SET_BUS_WIDTH = 0x06020000 | Self::CMD_NEED_APP.bits;
        const CMD_SEND_OP_COND = 0x29020000 | Self::CMD_NEED_APP.bits;
        const CMD_SEND_SCR = 0x33220010 | Self::CMD_NEED_APP.bits;
    }

    struct Status: u32 {
        const SR_READ_AVAILABLE = 0x00000800;
        const SR_DAT_INHIBIT = 0x00000002;
        const SR_CMD_INHIBIT = 0x00000001;
        const SR_APP_CMD = 0x00000020;
    }

    struct Interrupt: u32 {
        const INT_DATA_TIMEOUT = 0x00100000;
        const INT_CMD_TIMEOUT = 0x00010000;
        const INT_READ_RDY = 0x00000020;
        const INT_CMD_DONE = 0x00000001;
        const INT_ERROR_MASK = 0x017E8000;
    }

    struct Control: u32 {
        const C0_SPI_MODE_EN = 0x00100000;
        const C0_HCTL_HS_EN = 0x00000004;
        const C0_HCTL_DWITDH = 0x00000002;
        const C1_SRST_DATA = 0x04000000;
        const C1_SRST_HC = 0x01000000;
        const C1_TOUNIT_DIS = 0x000f0000;
        const C1_TOUNIT_MAX = 0x000e0000;
        const C1_CLK_GENSEL = 0x00000020;
        const C1_CLK_EN = 0x00000004;
        const C1_CLK_STABLE = 0x00000002;
        const C1_CLK_INTLEN = 0x00000001;
    }

    struct SlotISRVer: u32 {
        const HOST_SPEC_NUM = 0x00ff0000;
        const HOST_SPEC_NUM_SHIFT = 16;
        const HOST_SPEC_V3 = 2;
        const HOST_SPEC_V2 = 1;
        const HOST_SPEC_V1 = 0;
    }

    struct ScrFlag: u32 {
        const SCR_SD_BUS_WIDTH_4 = 0x00000400;
        const SCR_SUPP_SET_BLKCNT = 0x02000000;
        const SCR_SUPP_CCS = 0x00000001;
        const ACMD41_VOLTAGE = 0x00ff8000;
        const ACMD41_CMD_COMPLETE = 0x80000000;
        const ACMD41_CMD_CCS = 0x40000000;
        const ACMD41_ARG_HC = 0x51ff8000;
    }

    pub struct SDError: u32 {
        const SD_OK = 0;
        const SD_TIMEOUT = -1_i32 as u32;
        const SD_ERROR = -2_i32 as u32;
    }
}

#[allow(dead_code)]
pub struct SDCard {
    sd_scr: [u64; 2],
    sd_ocr: u64,
    sd_rca: u64,
    sd_err: u64,
    sd_hv: u64,
}

impl SDCard {
    fn read_reg(addr: usize) -> u32 {
        unsafe { (addr as *const u32).read_volatile() }
    }

    fn write_reg(addr: usize, data: u32) {
        unsafe { (addr as *mut u32).write_volatile(data) }
    }

    pub fn sd_status(&self, mask: u32) -> u32 {
        let mut count = 500000;
        while (Self::read_reg(EMMC_STATUS) & mask) != 0
            && Self::read_reg(EMMC_INTERRUPT) & Interrupt::INT_ERROR_MASK.bits() == 0
            && count != 0
        {
            count -= 1;
            wait_msec(1);
        }

        if count <= 0 || (Self::read_reg(EMMC_INTERRUPT) & Interrupt::INT_ERROR_MASK.bits()) != 0 {
            SDError::SD_ERROR.bits()
        } else {
            SDError::SD_OK.bits()
        }
    }

    pub fn sd_int(&self, mask: u32) -> u32 {
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
            return SDError::SD_TIMEOUT.bits();
        }
        if (r & Interrupt::INT_ERROR_MASK.bits()) != 0 {
            Self::write_reg(EMMC_INTERRUPT, r);
            return SDError::SD_ERROR.bits();
        }
        Self::write_reg(EMMC_INTERRUPT, mask);
        return SDError::SD_OK.bits();
    }

    pub fn sd_cmd(&mut self, mut code: u32, arg: u32) -> u32 {
        let mut r: u32 = 0;
        self.sd_err = SDError::SD_OK.bits() as u64;
        if (code & Command::CMD_NEED_APP.bits()) != 0 {
            let cmd = if self.sd_rca != 0 {
                Command::CMD_RSPNS_48.bits()
            } else {
                0
            };
            r = self.sd_cmd(Command::CMD_APP_CMD.bits() | cmd, self.sd_rca as u32);
            if self.sd_rca != 0 && r == 0 {
                error!("Failed to send SD APP command");
                self.sd_err = SDError::SD_ERROR.bits() as u64;
                return 0;
            }
            code &= !Command::CMD_NEED_APP.bits();
        }
        if self.sd_status(Status::SR_CMD_INHIBIT.bits()) != 0 {
            error!("EMMC busy");
            self.sd_err = SDError::SD_TIMEOUT.bits() as u64;
            return 0;
        }
        info!("Sending command: {:#x}, arg: {}", code, arg);
        Self::write_reg(EMMC_INTERRUPT, Self::read_reg(EMMC_INTERRUPT));
        Self::write_reg(EMMC_ARG1, arg);
        Self::write_reg(EMMC_CMDTM, code);
        if code == Command::CMD_SEND_OP_COND.bits() {
            wait_msec(1000);
        } else if code == Command::CMD_SEND_IF_COND.bits() || code == Command::CMD_APP_CMD.bits() {
            wait_msec(100);
        }
        r = self.sd_int(Interrupt::INT_CMD_DONE.bits());
        if r != 0 {
            error!("Failed to send EMMC command");
            self.sd_err = r as u64;
            return 0;
        }
        r = Self::read_reg(EMMC_RESP0);
        if code == Command::CMD_GO_IDLE.bits() || code == Command::CMD_APP_CMD.bits() {
            return 0;
        } else if code == (Command::CMD_APP_CMD | Command::CMD_RSPNS_48).bits() {
            return r & Status::SR_APP_CMD.bits();
        } else if code == Command::CMD_SEND_OP_COND.bits() {
            return r;
        } else if code == Command::CMD_SEND_IF_COND.bits() {
            return if r == arg {
                SDError::SD_OK.bits()
            } else {
                SDError::SD_ERROR.bits()
            };
        } else if code == Command::CMD_ALL_SEND_CID.bits() {
            r |= Self::read_reg(EMMC_RESP3);
            r |= Self::read_reg(EMMC_RESP2);
            r |= Self::read_reg(EMMC_RESP1);
            return r;
        } else if code == Command::CMD_SEND_REL_ADDR.bits() {
            self.sd_err =
                (((r & 0x1fff) | ((r & 0x2000) << 6) | ((r & 0x4000) << 8) | ((r & 0x8000) << 8))
                    & Command::CMD_ERRORS_MASK.bits()) as u64;
            return r & Command::CMD_RCA_MASK.bits();
        }

        r & Command::CMD_ERRORS_MASK.bits()
    }

    pub fn sd_readblock(&mut self, lba: u32, buffer: *mut u8, mut num: u32) -> u32 {
        let mut r = 0;
        let mut c = 0;
        // let d = 0;
        if num < 1 {
            num = 1;
        }
        info!("sd_readblock lba: {}, num: {}", lba, num);
        if self.sd_status(Status::SR_DAT_INHIBIT.bits()) != 0 {
            self.sd_err = SDError::SD_TIMEOUT.bits() as u64;
            return 0;
        }
        let mut buf: *mut u32 = buffer as *mut u32;
        if self.sd_scr[0] & (ScrFlag::SCR_SUPP_CCS.bits() as u64) != 0 {
            if num > 1 && (self.sd_scr[0] & (ScrFlag::SCR_SUPP_SET_BLKCNT.bits() as u64)) != 0 {
                self.sd_cmd(Command::CMD_SET_BLOCKCNT.bits(), num);
                if self.sd_err != 0 {
                    return 0;
                }
            }
            Self::write_reg(EMMC_BLKSIZECNT, (num << 16) | 512);
            self.sd_cmd(
                if num == 1 {
                    Command::CMD_READ_SINGLE.bits()
                } else {
                    Command::CMD_READ_MULTI.bits()
                },
                lba,
            );
            if self.sd_err != 0 {
                return 0;
            }
        } else {
            Self::write_reg(EMMC_BLKSIZECNT, (1 << 16) | 512);
        }
        while c < num {
            if (self.sd_scr[0] & (ScrFlag::SCR_SUPP_CCS.bits() as u64)) == 0 {
                self.sd_cmd(Command::CMD_READ_SINGLE.bits(), (lba + c) * 512);
                if self.sd_err != 0 {
                    return 0;
                }
            }
            r = self.sd_int(Interrupt::INT_READ_RDY.bits());
            if r != 0 {
                error!("Timeout waiting for ready to read");
                self.sd_err = r as u64;
                return 0;
            }
            for d in 0..128 {
                unsafe {
                    buf.add(d).write_volatile(Self::read_reg(EMMC_DATA));
                }
            }
            unsafe {
                c += 1;
                buf = buf.add(128);
            }
        }
        if num > 1
            && (self.sd_scr[0] & (ScrFlag::SCR_SUPP_SET_BLKCNT.bits() as u64)) == 0
            && (self.sd_scr[0] & (ScrFlag::SCR_SUPP_CCS.bits() as u64)) != 0
        {
            self.sd_cmd(Command::CMD_STOP_TRANS.bits(), 0);
        }
        return if self.sd_err != (SDError::SD_OK.bits() as u64) || c != num {
            0
        } else {
            num * 512
        };
    }

    pub fn sd_clk(&mut self, f: u32) -> u32 {
        let mut d: u32 = 0;
        let c: u32 = 41666666 / f;
        let mut x: u32 = 0;
        let mut s: u32 = 32;
        let mut h: u32 = 0;
        let mut cnt: i32 = 100000;
        while (Self::read_reg(EMMC_STATUS)
            & (Status::SR_CMD_INHIBIT.bits() | Status::SR_DAT_INHIBIT.bits()))
            != 0
            && cnt != 0
        {
            cnt -= 1;
            wait_msec(1);
        }
        if cnt <= 0 {
            error!("Timeout waiting for inhibit flag");
            return SDError::SD_ERROR.bits();
        }
        Self::write_reg(
            EMMC_CONTROL1,
            Self::read_reg(EMMC_CONTROL1) & !(Control::C1_CLK_EN.bits()),
        );
        wait_msec(10);
        x = c - 1;
        if x == 0 {
            s = 0;
        } else {
            if x & 0xffff_0000 == 0 {
                x <<= 16;
                s -= 16;
            }
            if x & 0xff00_0000 == 0 {
                x <<= 8;
                s -= 8;
            }
            if x & 0xf000_0000 == 0 {
                x <<= 4;
                s -= 4;
            }
            if x & 0xc000_0000 == 0 {
                x <<= 2;
                s -= 2;
            }
            if x & 0x8000_0000 == 0 {
                x <<= 1;
                s -= 1;
            }
            if s > 0 {
                s -= 1;
            }
            if s > 7 {
                s = 7;
            }
        }
        if self.sd_hv > (SlotISRVer::HOST_SPEC_V2.bits() as u64) {
            d = c;
        } else {
            d = 1 << s;
        }
        if d <= 2 {
            d = 2;
            s = 0;
        }
        info!("sd_clk divisor: {:x}, shift: {:x}", d, s);
        if self.sd_hv > (SlotISRVer::HOST_SPEC_V2.bits() as u64) {
            h = (d & 0x300) >> 2;
        }
        d = ((d & 0x0ff) << 8) | h;
        Self::write_reg(
            EMMC_CONTROL1,
            (Self::read_reg(EMMC_CONTROL1) & 0xffff_003f) | d,
        );
        wait_msec(10);
        Self::write_reg(
            EMMC_CONTROL1,
            Self::read_reg(EMMC_CONTROL1) | Control::C1_CLK_EN.bits(),
        );
        wait_msec(10);
        cnt = 10000;
        while ((Self::read_reg(EMMC_CONTROL1) & Control::C1_CLK_STABLE.bits()) == 0) && cnt != 0 {
            cnt -= 1;
            wait_msec(10);
        }
        if cnt <= 0 {
            error!("Failed to get stable clock");
            return SDError::SD_ERROR.bits();
        }
        return SDError::SD_OK.bits();
    }

    pub fn sd_init(&mut self) -> u32 {
        let mut r: i64;
        let mut cnt: i64;
        let mut ccs: i64 = 0;
        r = Self::read_reg(GPFSEL4) as i64;
        r &= !(7 << (7 * 3));
        Self::write_reg(GPFSEL4, r as u32);
        Self::write_reg(GPPUD, 2);
        wait_cycles(150);
        Self::write_reg(GPPUDCLK1, 1 << 15);
        wait_cycles(150);
        Self::write_reg(GPPUD, 0);
        Self::write_reg(GPPUDCLK1, 0);
        r = Self::read_reg(GPHEN1) as i64;
        r |= 1 << 15;
        Self::write_reg(GPHEN1, r as u32);

        r = Self::read_reg(GPFSEL4) as i64;
        r |= (7 << (8 * 3)) | (7 << (9 * 3));
        Self::write_reg(GPFSEL4, r as u32);
        Self::write_reg(GPPUD, 2);
        wait_cycles(150);
        Self::write_reg(GPPUDCLK1, (1 << 16) | (1 << 17));
        wait_cycles(150);
        Self::write_reg(GPPUD, 0);
        Self::write_reg(GPPUDCLK1, 0);

        r = Self::read_reg(GPFSEL5) as i64;
        r |= (7 << (0 * 3)) | (7 << (1 * 3)) | (7 << (2 * 3)) | (7 << (3 * 3));
        Self::write_reg(GPFSEL5, r as u32);
        Self::write_reg(GPPUD, 2);
        wait_cycles(150);
        Self::write_reg(GPPUDCLK1, (1 << 18) | (1 << 19) | (1 << 20) | (1 << 21));
        wait_cycles(150);
        Self::write_reg(GPPUD, 0);
        Self::write_reg(GPPUDCLK1, 0);

        self.sd_hv = ((Self::read_reg(EMMC_SLOTISR_VER) & SlotISRVer::HOST_SPEC_NUM.bits()) as u64)
            >> SlotISRVer::HOST_SPEC_NUM_SHIFT.bits();
        info!("EMMC: GPIO set up");
        Self::write_reg(EMMC_CONTROL0, 0);
        Self::write_reg(
            EMMC_CONTROL1,
            Self::read_reg(EMMC_CONTROL1) | Control::C1_SRST_HC.bits(),
        );
        cnt = 10000;
        while (Self::read_reg(EMMC_CONTROL1) & Control::C1_SRST_HC.bits()) != 0 && cnt != 0 {
            cnt -= 1;
            wait_msec(10);
        }
        if cnt <= 0 {
            error!("Failed to reset EMMC");
            return SDError::SD_ERROR.bits();
        }
        info!("EMMC: reset OK");
        Self::write_reg(
            EMMC_CONTROL1,
            Self::read_reg(EMMC_CONTROL1)
                | Control::C1_CLK_INTLEN.bits()
                | Control::C1_TOUNIT_MAX.bits(),
        );
        wait_msec(10);
        r = self.sd_clk(400000) as i64;
        if r != 0 {
            return r as u32;
        }
        Self::write_reg(EMMC_INT_EN, 0xffff_ffff);
        Self::write_reg(EMMC_INT_MASK, 0xffff_ffff);
        self.sd_scr[0] = 0;
        self.sd_scr[1] = 0;
        self.sd_rca = 0;
        self.sd_err = 0;
        self.sd_cmd(Command::CMD_GO_IDLE.bits(), 0);
        if self.sd_err != 0 {
            return self.sd_err as u32;
        }

        self.sd_cmd(Command::CMD_SEND_IF_COND.bits(), 0x0000_01AA);
        if self.sd_err != 0 {
            return self.sd_err as u32;
        }
        cnt = 6;
        r = 0;
        while (r & (ScrFlag::ACMD41_CMD_COMPLETE.bits() as i64)) == 0 && cnt != 0 {
            cnt -= 1;
            wait_cycles(400);
            r = self.sd_cmd(
                Command::CMD_SEND_OP_COND.bits(),
                ScrFlag::ACMD41_ARG_HC.bits(),
            ) as i64;
            info!("EMMC: CMD_SEND_OP_COND returned");
            if r & (ScrFlag::ACMD41_CMD_COMPLETE.bits() as i64) != 0 {
                info!("COMPLETE: ");
            }
            if (r & (ScrFlag::ACMD41_VOLTAGE.bits() as i64)) != 0 {
                info!("VOLTAGE: ");
            }
            if (r & (ScrFlag::ACMD41_CMD_CCS.bits() as i64)) != 0 {
                info!("CCS: ");
            }
            info!("{}", r as usize >> 32);
            info!("{}", r);
            if self.sd_err != (SDError::SD_TIMEOUT.bits() as u64)
                && self.sd_err != (SDError::SD_OK.bits() as u64)
            {
                info!("EMMC ACMD41 returned error");
                return self.sd_err as u32;
            }
        }
        if (r & (ScrFlag::ACMD41_CMD_COMPLETE.bits() as i64)) == 0 || cnt == 0 {
            return SDError::SD_TIMEOUT.bits();
        }
        if (r & (ScrFlag::ACMD41_VOLTAGE.bits() as i64)) == 0 {
            return SDError::SD_ERROR.bits();
        }
        if (r & (ScrFlag::ACMD41_CMD_CCS.bits() as i64)) != 0 {
            ccs = ScrFlag::SCR_SUPP_CCS.bits() as i64;
        }
        self.sd_cmd(Command::CMD_ALL_SEND_CID.bits(), 0);
        self.sd_rca = self.sd_cmd(Command::CMD_SEND_REL_ADDR.bits(), 0) as u64;
        info!(
            "EMMC: CMD_SEND_REL_ADDR returned: {}, {}",
            self.sd_rca as usize >> 32,
            self.sd_rca
        );
        if self.sd_err != 0 {
            return self.sd_err as u32;
        }

        r = self.sd_clk(2500_0000) as i64;
        if r != 0 {
            return r as u32;
        }

        self.sd_cmd(Command::CMD_CARD_SELECT.bits(), self.sd_rca as u32);
        if self.sd_err != 0 {
            return self.sd_err as u32;
        }

        if self.sd_status(Status::SR_DAT_INHIBIT.bits()) != 0 {
            return SDError::SD_TIMEOUT.bits();
        }
        Self::write_reg(EMMC_BLKSIZECNT, (1 << 16) | 8);
        self.sd_cmd(Command::CMD_SEND_SCR.bits(), 0);
        if self.sd_err != 0 {
            return self.sd_err as u32;
        }
        if self.sd_int(Interrupt::INT_READ_RDY.bits()) != 0 {
            return SDError::SD_TIMEOUT.bits();
        }

        r = 0;
        cnt = 100000;
        while r < 2 && cnt != 0 {
            if Self::read_reg(EMMC_STATUS) & Status::SR_READ_AVAILABLE.bits() != 0 {
                self.sd_scr[r as usize] = Self::read_reg(EMMC_DATA) as u64;
                r += 1;
            } else {
                wait_msec(1);
            }
        }

        if r != 2 {
            return SDError::SD_TIMEOUT.bits();
        }
        if self.sd_scr[0] & (ScrFlag::SCR_SD_BUS_WIDTH_4.bits() as u64) != 0 {
            self.sd_cmd(Command::CMD_SET_BUS_WIDTH.bits(), (self.sd_rca as u32) | 2);
            if self.sd_err != 0 {
                return self.sd_err as u32;
            }
            Self::write_reg(
                EMMC_CONTROL0,
                Self::read_reg(EMMC_CONTROL0) | Control::C0_HCTL_DWITDH.bits(),
            );
        }
        info!(
            "EMMC: supports: {}",
            if self.sd_scr[0] & (ScrFlag::SCR_SUPP_SET_BLKCNT.bits() as u64) != 0 {
                "SET_BLKCNT"
            } else {
                "CCS"
            }
        );
        self.sd_scr[0] &= !(ScrFlag::SCR_SUPP_CCS.bits() as u64);
        self.sd_scr[0] |= ccs as u64;
        return SDError::SD_OK.bits();
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

pub fn wait_cycles(mut n: u32) {
    if n != 0 {
        while n != 0 {
            n -= 1;
            unsafe {
                asm!("nop");
            }
        }
    }
}
