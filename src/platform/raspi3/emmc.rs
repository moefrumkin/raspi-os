use crate::volatile::Volatile;
use crate::bitfield;

#[repr(C)]
pub struct EMMCController {
    pub arg2: Volatile<u32>,
    pub blockSizeAndCount: Volatile<BlockSizeAndCount>,
    pub arg1: Volatile<u32>,
    pub cmdtm: Volatile<CMDTM>,
    pub resp0: Volatile<u32>,
    pub resp1: Volatile<u32>,
    pub resp2: Volatile<u32>,
    pub resp3: Volatile<u32>,
    pub data: Volatile<u32>,
    pub status: Volatile<Status>,
    pub control0: Volatile<Control0>,
    pub control1: Volatile<Control1>,
    pub interrupt: Volatile<Interrupt>,
    pub irpt_mask: Volatile<InterruptMask>,
    pub irpt_en: Volatile<InterruptEnable>,
    pub control2: Volatile<u32>,
    pub force_irpt: Volatile<ForceInterrupt>,
    pub boot_timeout: Volatile<u32>,
    pub dbg_sgl: Volatile<DBG_SEL>,
    pub exrdfifo_cfg: Volatile<EXRDFIFO_CFG>,
    pub exrdfifo_en: Volatile<EXRDFIFO_EN>,
    pub tune_step: Volatile<TUNE_STEP>,
    pub tune_steps_std: Volatile<TUNE_STEPS_STD>,
    pub tune_steps_ddr: Volatile<TUNE_STEPS_DDR>,
    pub spi_int_spt: Volatile<SPI_INT_SPT>,
    pub slotisr_ver: Volatile<SLOTISR_VER>
}

bitfield! {
    BlockSizeAndCount(u32) {
        blockSize: 0-9,
        numberOfBlocks: 16-31
    }
}

bitfield! {
    CMDTM(u32) {
        TM_BLKCNT_EN: 1-1,
        TM_AUTO_CMD_END: 2-3,
        TM_DAT_DIR: 4-4,
        TM_MULTI_BLOCK: 5-5,
        CMD_RSPNS_TYPE: 16-17,
        CMD_CRCCHK_EN: 19-19,
        CMD_IXCHK_EN: 20-20,
        CMD_ISDATA: 21-21,
        CMD_TYPE: 22-23,
        CMD_INDEX: 24-29
    }
}

bitfield! {
    Status(u32) {
        CMD_INHIBIT: 0-0,
        DAT_INHIBIT: 1-1,
        DAT_ACTIVE: 2-2,
        WRITE_TRANSFER: 8-8,
        READ_TRANSFER: 9-9,
        DAT_LEVEL0: 20-23,
        CMD_LEVEL: 24-24,
        DAT_LEVEL1: 25-28
    }
}

bitfield! {
    Control0(u32) {
        HCTL_DWIDTH: 1-1,
        HCTL_HS_EN: 2-2,
        HCTL_8BIT: 5-5,
        GAP_STOP: 16-16,
        GAP_RESTART: 17-17,
        READWAIT_EN: 18-18,
        GAP_IEN: 19-19,
        SPI_MODE: 20-20,
        BOOT_EN: 21-21,
        ALT_BOOT_EN: 22-22
    }
}

bitfield! {
    Control1(u32) {
        CLK_INTLEN: 0-0,
        CLK_STABLE: 1-1,
        CLK_EN: 2-2,
        CLK_GENSEL: 5-5,
        CLK_FREQ_MS2: 6-7,
        CLK_FREQ8: 8-15,
        DATA_TOUNIT: 16-19,
        SRST_HC: 24-24,
        SRST_CMD: 25-25,
        SRST_DATA: 26-26
    }
}

bitfield! {
    Interrupt(u32) {
        CMD_DONE: 0-0,
        DATA_DONE: 1-1,
        BLOCK_GAP: 2-2,
        WRITE_RDY: 4-4,
        READ_RDY: 5-5,
        CARD: 8-8,
        RETUNE: 12-12,
        BOOTACK: 13-13,
        ENDBOOT: 14-14,
        ERR: 15-15,
        CTO_ERR: 16-16,
        CCRC_ERR: 17-17,
        CEND_ERR: 18-18,
        CBAD_ERR: 19-19,
        DTO_ERR: 20-20,
        DCRC_ERR: 21-21,
        DEND_ERR: 22-22,
        ACMD_ERR: 24-24
    }
}

bitfield! {
    InterruptMask(u32) {
        CMD_DONE: 0-0,
        DATA_DONE: 1-1,
        BLOCK_GAP: 2-2,
        WRITE_RDY: 4-4,
        READ_RDY: 5-5,
        CARD: 8-8,
        RETUNE: 12-12,
        BOOTACK: 13-13,
        ENDBOOT: 14-14,
        CTO_ERR: 16-16,
        CRRC_ERR: 17-17,
        CBAD_ERR: 19-19,
        DTO_ERR: 20-20,
        DCRC_ERR: 21-21,
        DEND_ERR: 22-22,
        ACMD_ERR: 24-24
    }
}

bitfield! {
    InterruptEnable(u32) {
        CMD_DONE: 0-0,
        DATA_DONE: 1-1,
        BLOCK_GAP: 2-2,
        WRITE_RDY: 4-4,
        READ_RDY: 5-5,
        CARD: 8-8,
        RETUNE: 12-12,
        BOOTACK: 13-13,
        ENDBOOT: 14-14,
        CTO_ERR: 16-16,
        CRRC_ERR: 17-17,
        CBAD_ERR: 19-19,
        DTO_ERR: 20-20,
        DCRC_ERR: 21-21,
        DEND_ERR: 22-22,
        ACMD_ERR: 24-24
    }
}

bitfield! {
    Control2(u32) {
        ACNOX_ERR: 0-0,
        ACTO_ERR: 1-1,
        ACCRC_ERR: 2-2,
        ACEND_ERR: 3-3,
        ACBAD_ERR: 4-4,
        NOTC12_ERR: 7-7,
        UHSMODE: 16-18,
        TUNEON: 22-22,
        TUNED: 23-23
    }
}

bitfield! {
    ForceInterrupt(u32) {
        CMD_DONE: 0-0,
        DATA_DONE: 1-1,
        BLOCK_GAP: 2-2,
        WRITE_RDY: 4-4,
        READ_RDY: 5-5,
        CARD: 8-8,
        RETUNE: 12-12,
        BOOTACK: 13-13,
        ENDBOOT: 14-14,
        CTO_ERR: 16-16,
        CRRC_ERR: 17-17,
        CBAD_ERR: 19-19,
        DTO_ERR: 20-20,
        DCRC_ERR: 21-21,
        DEND_ERR: 22-22,
        ACMD_ERR: 24-24
    }
}

bitfield! {
    DBG_SEL(u32) {
        SELECT: 0-0
    }
}

bitfield! {
    EXRDFIFO_CFG(u32) {
        RD_THRSH: 0-2
    }
}

bitfield! {
    EXRDFIFO_EN(u32) {
        ENABLE: 0-0
    }
}

bitfield! {
    TUNE_STEP(u32) {
        DELAY: 0-2
    }
}

bitfield! {
    TUNE_STEPS_STD(u32) {
        STEPS: 0-5
    }
}

bitfield! {
    TUNE_STEPS_DDR(u32) {
        STEPS: 0-5
    }
}

bitfield! {
    SPI_INT_SPT(u32) {
        SELECT: 0-7
    }
}

bitfield! {
    SLOTISR_VER(u32) {
        VENDOR: 24-31,
        SDVERSION: 16-23,
        SLOT_STATUS: 0-7
    }
}