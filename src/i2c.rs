use core::ops::Deref;

use embedded_hal::blocking::i2c::{Read, Write, WriteRead};

use crate::{
    gpio::*,
    rcc::Rcc,
    time::{KiloHertz, U32Ext},
};

/// I2C abstraction
pub struct I2c<I2C, SCLPIN, SDAPIN> {
    i2c: I2C,
    pins: (SCLPIN, SDAPIN),
}

pub trait SclPin<I2C> {}
pub trait SdaPin<I2C> {}

macro_rules! i2c_pins {
    ($($I2C:ident => {
        scl => [$($scl:ty),+ $(,)*],
        sda => [$($sda:ty),+ $(,)*],
    })+) => {
        $(
            $(
                impl SclPin<crate::pac::$I2C> for $scl {}
            )+
            $(
                impl SdaPin<crate::pac::$I2C> for $sda {}
            )+
        )+
    }
}

#[cfg(any(
    feature = "stm32f030",
    feature = "stm32f031",
    feature = "stm32f038",
    feature = "stm32f042",
    feature = "stm32f048",
    feature = "stm32f051",
    feature = "stm32f058",
    feature = "stm32f070",
    feature = "stm32f071",
    feature = "stm32f072",
    feature = "stm32f078",
    feature = "stm32f091",
    feature = "stm32f098",
))]
i2c_pins! {
    I2C1 => {
        scl => [gpiob::PB6<Alternate<AF1>>, gpiob::PB8<Alternate<AF1>>],
        sda => [gpiob::PB7<Alternate<AF1>>, gpiob::PB9<Alternate<AF1>>],
    }
}
#[cfg(any(
    feature = "stm32f030x4",
    feature = "stm32f030x6",
    feature = "stm32f030xc",
    feature = "stm32f031",
    feature = "stm32f038",
    feature = "stm32f042",
    feature = "stm32f048",
    feature = "stm32f070x6",
    feature = "stm32f091",
    feature = "stm32f098",
))]
i2c_pins! {
    I2C1 => {
        scl => [gpioa::PA9<Alternate<AF4>>],
        sda => [gpioa::PA10<Alternate<AF4>>],
    }
}
#[cfg(any(feature = "stm32f042", feature = "stm32f048"))]
i2c_pins! {
    I2C1 => {
        scl => [gpioa::PA11<Alternate<AF5>>],
        sda => [gpioa::PA12<Alternate<AF5>>],
    }
}
#[cfg(any(
    feature = "stm32f030x4",
    feature = "stm32f030x6",
    feature = "stm32f031",
    feature = "stm32f038",
    feature = "stm32f042",
    feature = "stm32f048",
    feature = "stm32f072",
))]
i2c_pins! {
    I2C1 => {
        scl => [gpiob::PB10<Alternate<AF1>>],
        sda => [gpiob::PB11<Alternate<AF1>>],
    }
}
#[cfg(any(feature = "stm32f030xc", feature = "stm32f042", feature = "stm32f048"))]
i2c_pins! {
    I2C1 => {
        scl => [gpiob::PB13<Alternate<AF5>>],
        sda => [gpiob::PB14<Alternate<AF5>>],
    }
}
#[cfg(any(
    feature = "stm32f030xc",
    feature = "stm32f042",
    feature = "stm32f048",
    feature = "stm32f070x6",
    feature = "stm32f091",
    feature = "stm32f098",
))]
i2c_pins! {
    I2C1 => {
        scl => [gpiof::PF1<Alternate<AF1>>],
        sda => [gpiof::PF0<Alternate<AF1>>],
    }
}

#[cfg(any(feature = "stm32f030x8", feature = "stm32f051", feature = "stm32f058"))]
i2c_pins! {
    I2C2 => {
        scl => [gpiob::PB10<Alternate<AF1>>],
        sda => [gpiob::PB11<Alternate<AF1>>],
    }
}
#[cfg(any(
    feature = "stm32f030xc",
    feature = "stm32f070xb",
    feature = "stm32f071",
    feature = "stm32f072",
    feature = "stm32f078",
    feature = "stm32f091",
    feature = "stm32f098",
))]
i2c_pins! {
    I2C2 => {
        scl => [gpiob::PB10<Alternate<AF1>>, gpiob::PB13<Alternate<AF5>>],
        sda => [gpiob::PB11<Alternate<AF1>>, gpiob::PB14<Alternate<AF5>>],
    }
}
#[cfg(any(feature = "stm32f091", feature = "stm32f098"))]
i2c_pins! {
    I2C2 => {
        scl => [gpioa::PA11<Alternate<AF5>>],
        sda => [gpioa::PA12<Alternate<AF5>>],
    }
}

#[derive(Debug)]
pub enum Error {
    OVERRUN,
    NACK,
    BUS,
}

macro_rules! i2c {
    ($($I2C:ident: ($i2c:ident, $i2cXen:ident, $i2cXrst:ident, $apbenr:ident, $apbrstr:ident),)+) => {
        $(
            use crate::pac::$I2C;
            impl<SCLPIN, SDAPIN> I2c<$I2C, SCLPIN, SDAPIN> {
                pub fn $i2c(i2c: $I2C, pins: (SCLPIN, SDAPIN), speed: KiloHertz, rcc: &mut Rcc) -> Self
                where
                    SCLPIN: SclPin<$I2C>,
                    SDAPIN: SdaPin<$I2C>,
                {
                    // Enable clock for I2C
                    rcc.regs.$apbenr.modify(|_, w| w.$i2cXen().set_bit());

                    // Reset I2C
                    rcc.regs.$apbrstr.modify(|_, w| w.$i2cXrst().set_bit());
                    rcc.regs.$apbrstr.modify(|_, w| w.$i2cXrst().clear_bit());
                    I2c { i2c, pins }.i2c_init(speed)
                }
            }
        )+
    }
}

i2c! {
    I2C1: (i2c1, i2c1en, i2c1rst, apb1enr, apb1rstr),
}

#[cfg(any(
    feature = "stm32f030x8",
    feature = "stm32f030xc",
    feature = "stm32f051",
    feature = "stm32f058",
    feature = "stm32f070xb",
    feature = "stm32f071",
    feature = "stm32f072",
    feature = "stm32f078",
    feature = "stm32f091",
    feature = "stm32f098",
))]
i2c! {
    I2C2: (i2c2, i2c2en, i2c2rst, apb1enr, apb1rstr),
}

// It's s needed for the impls, but rustc doesn't recognize that
#[allow(dead_code)]
type I2cRegisterBlock = crate::pac::i2c1::RegisterBlock;

impl<I2C, SCLPIN, SDAPIN> I2c<I2C, SCLPIN, SDAPIN>
where
    I2C: Deref<Target = I2cRegisterBlock>,
{
    fn i2c_init(self, speed: KiloHertz) -> Self {
        use core::cmp;

        // Make sure the I2C unit is disabled so we can configure it
        self.i2c.cr1.modify(|_, w| w.pe().clear_bit());

        // Calculate settings for I2C speed modes
        let presc;
        let scldel;
        let sdadel;
        let sclh;
        let scll;

        // We're using HSI here which runs at a fixed 8MHz
        const FREQ: u32 = 8_000_000;

        // Normal I2C speeds use a different scaling than fast mode below
        if speed <= 100_u32.khz() {
            presc = 1;
            scll = cmp::max((((FREQ >> presc) >> 1) / speed.0) - 1, 255) as u8;
            sclh = scll - 4;
            sdadel = 2;
            scldel = 4;
        } else {
            presc = 0;
            scll = cmp::max((((FREQ >> presc) >> 1) / speed.0) - 1, 255) as u8;
            sclh = scll - 6;
            sdadel = 1;
            scldel = 3;
        }

        // Enable I2C signal generator, and configure I2C for configured speed
        self.i2c.timingr.write(|w| {
            w.presc()
                .bits(presc)
                .scldel()
                .bits(scldel)
                .sdadel()
                .bits(sdadel)
                .sclh()
                .bits(sclh)
                .scll()
                .bits(scll)
        });

        // Enable the I2C processing
        self.i2c.cr1.modify(|_, w| w.pe().set_bit());

        self
    }

    pub fn release(self) -> (I2C, (SCLPIN, SDAPIN)) {
        (self.i2c, self.pins)
    }

    fn check_and_clear_error_flags(&self, isr: &crate::stm32::i2c1::isr::R) -> Result<(), Error> {
        // If we have a set overrun flag, clear it and return an OVERRUN error
        if isr.ovr().bit_is_set() {
            self.i2c.icr.write(|w| w.ovrcf().set_bit());
            return Err(Error::OVERRUN);
        }

        // If we have a set arbitration error or bus error flag, clear it and return an BUS error
        if isr.arlo().bit_is_set() | isr.berr().bit_is_set() {
            self.i2c
                .icr
                .write(|w| w.arlocf().set_bit().berrcf().set_bit());
            return Err(Error::BUS);
        }

        // If we received a NACK, then signal as a NACK error
        if isr.nackf().bit_is_set() {
            self.i2c
                .icr
                .write(|w| w.stopcf().set_bit().nackcf().set_bit());
            return Err(Error::NACK);
        }

        Ok(())
    }

    fn send_byte(&self, byte: u8) -> Result<(), Error> {
        // Wait until we're ready for sending
        loop {
            let isr = self.i2c.isr.read();
            self.check_and_clear_error_flags(&isr)?;
            if isr.txis().bit_is_set() {
                break;
            }
        }

        // Push out a byte of data
        self.i2c.txdr.write(|w| unsafe { w.bits(u32::from(byte)) });

        self.check_and_clear_error_flags(&self.i2c.isr.read())?;
        Ok(())
    }

    fn recv_byte(&self) -> Result<u8, Error> {
        loop {
            let isr = self.i2c.isr.read();
            self.check_and_clear_error_flags(&isr)?;
            if isr.rxne().bit_is_set() {
                break;
            }
        }

        let value = self.i2c.rxdr.read().bits() as u8;
        Ok(value)
    }
}

impl<I2C, SCLPIN, SDAPIN> WriteRead for I2c<I2C, SCLPIN, SDAPIN>
where
    I2C: Deref<Target = I2cRegisterBlock>,
{
    type Error = Error;

    fn write_read(&mut self, addr: u8, bytes: &[u8], buffer: &mut [u8]) -> Result<(), Error> {
        // Set up current slave address for writing and disable autoending
        self.i2c.cr2.modify(|_, w| {
            w.sadd()
                .bits(u16::from(addr) << 1)
                .nbytes()
                .bits(bytes.len() as u8)
                .rd_wrn()
                .clear_bit()
                .autoend()
                .clear_bit()
        });

        // Send a START condition
        self.i2c.cr2.modify(|_, w| w.start().set_bit());

        // Wait until the transmit buffer is empty and there hasn't been any error condition
        loop {
            let isr = self.i2c.isr.read();
            self.check_and_clear_error_flags(&isr)?;
            if isr.txis().bit_is_set() || isr.tc().bit_is_set() {
                break;
            }
        }

        // Send out all individual bytes
        for c in bytes {
            self.send_byte(*c)?;
        }

        // Wait until data was sent
        loop {
            let isr = self.i2c.isr.read();
            self.check_and_clear_error_flags(&isr)?;
            if isr.tc().bit_is_set() {
                break;
            }
        }

        // Set up current address for reading
        self.i2c.cr2.modify(|_, w| {
            w.sadd()
                .bits(u16::from(addr) << 1)
                .nbytes()
                .bits(buffer.len() as u8)
                .rd_wrn()
                .set_bit()
        });

        // Send another START condition
        self.i2c.cr2.modify(|_, w| w.start().set_bit());

        // Send the autoend after setting the start to get a restart
        self.i2c.cr2.modify(|_, w| w.autoend().set_bit());

        // Now read in all bytes
        for c in buffer.iter_mut() {
            *c = self.recv_byte()?;
        }

        // Check and clear flags if they somehow ended up set
        self.check_and_clear_error_flags(&self.i2c.isr.read())?;

        Ok(())
    }
}

impl<I2C, SCLPIN, SDAPIN> Read for I2c<I2C, SCLPIN, SDAPIN>
where
    I2C: Deref<Target = I2cRegisterBlock>,
{
    type Error = Error;

    fn read(&mut self, addr: u8, buffer: &mut [u8]) -> Result<(), Error> {
        // Set up current address for reading
        self.i2c.cr2.modify(|_, w| {
            w.sadd()
                .bits(u16::from(addr) << 1)
                .nbytes()
                .bits(buffer.len() as u8)
                .rd_wrn()
                .set_bit()
        });

        // Send a START condition
        self.i2c.cr2.modify(|_, w| w.start().set_bit());

        // Send the autoend after setting the start to get a restart
        self.i2c.cr2.modify(|_, w| w.autoend().set_bit());

        // Now read in all bytes
        for c in buffer.iter_mut() {
            *c = self.recv_byte()?;
        }

        // Check and clear flags if they somehow ended up set
        self.check_and_clear_error_flags(&self.i2c.isr.read())?;

        Ok(())
    }
}

impl<I2C, SCLPIN, SDAPIN> Write for I2c<I2C, SCLPIN, SDAPIN>
where
    I2C: Deref<Target = I2cRegisterBlock>,
{
    type Error = Error;

    fn write(&mut self, addr: u8, bytes: &[u8]) -> Result<(), Error> {
        // Set up current slave address for writing and enable autoending
        self.i2c.cr2.modify(|_, w| {
            w.sadd()
                .bits(u16::from(addr) << 1)
                .nbytes()
                .bits(bytes.len() as u8)
                .rd_wrn()
                .clear_bit()
                .autoend()
                .set_bit()
        });

        // Send a START condition
        self.i2c.cr2.modify(|_, w| w.start().set_bit());

        // Send out all individual bytes
        for c in bytes {
            self.send_byte(*c)?;
        }

        // Check and clear flags if they somehow ended up set
        self.check_and_clear_error_flags(&self.i2c.isr.read())?;

        Ok(())
    }
}
