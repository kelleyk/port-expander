//! Support for the PCA9555 "16-bit I2C-bus and SMBus I/O port with interrupt"
use crate::I2cExt;

pub struct Pca9555<M>(M);

impl<I2C> Pca9555<shared_bus::NullMutex<Driver<I2C>>>
where
    I2C: crate::I2cBus,
{
    pub fn new(i2c: I2C) -> Self {
        Self::with_mutex(i2c)
    }
}

impl<I2C, M> Pca9555<M>
where
    I2C: crate::I2cBus,
    M: shared_bus::BusMutex<Bus = Driver<I2C>>,
{
    pub fn with_mutex(i2c: I2C) -> Self {
        Self(shared_bus::BusMutex::create(Driver::new(i2c)))
    }

    pub fn split<'a>(&'a mut self) -> Parts<'a, I2C, M> {
        Parts {
            io0_0: crate::Pin::new(0, &self.0),
            io0_1: crate::Pin::new(1, &self.0),
            io0_2: crate::Pin::new(2, &self.0),
            io0_3: crate::Pin::new(3, &self.0),
            io0_4: crate::Pin::new(4, &self.0),
            io0_5: crate::Pin::new(5, &self.0),
            io0_6: crate::Pin::new(6, &self.0),
            io0_7: crate::Pin::new(7, &self.0),
            io1_0: crate::Pin::new(8, &self.0),
            io1_1: crate::Pin::new(9, &self.0),
            io1_2: crate::Pin::new(10, &self.0),
            io1_3: crate::Pin::new(11, &self.0),
            io1_4: crate::Pin::new(12, &self.0),
            io1_5: crate::Pin::new(13, &self.0),
            io1_6: crate::Pin::new(14, &self.0),
            io1_7: crate::Pin::new(15, &self.0),
        }
    }
}

pub struct Parts<'a, I2C, M = shared_bus::NullMutex<Driver<I2C>>>
where
    I2C: crate::I2cBus,
    M: shared_bus::BusMutex<Bus = Driver<I2C>>,
{
    pub io0_0: crate::Pin<'a, crate::mode::Input, M>,
    pub io0_1: crate::Pin<'a, crate::mode::Input, M>,
    pub io0_2: crate::Pin<'a, crate::mode::Input, M>,
    pub io0_3: crate::Pin<'a, crate::mode::Input, M>,
    pub io0_4: crate::Pin<'a, crate::mode::Input, M>,
    pub io0_5: crate::Pin<'a, crate::mode::Input, M>,
    pub io0_6: crate::Pin<'a, crate::mode::Input, M>,
    pub io0_7: crate::Pin<'a, crate::mode::Input, M>,
    pub io1_0: crate::Pin<'a, crate::mode::Input, M>,
    pub io1_1: crate::Pin<'a, crate::mode::Input, M>,
    pub io1_2: crate::Pin<'a, crate::mode::Input, M>,
    pub io1_3: crate::Pin<'a, crate::mode::Input, M>,
    pub io1_4: crate::Pin<'a, crate::mode::Input, M>,
    pub io1_5: crate::Pin<'a, crate::mode::Input, M>,
    pub io1_6: crate::Pin<'a, crate::mode::Input, M>,
    pub io1_7: crate::Pin<'a, crate::mode::Input, M>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Regs {
    InputPort0 = 0x00,
    InputPort1 = 0x01,
    OutputPort0 = 0x02,
    OutputPort1 = 0x03,
    PolarityInversion0 = 0x04,
    PolarityInversion1 = 0x05,
    Configuration0 = 0x06,
    Configuration1 = 0x07,
}

impl From<Regs> for u8 {
    fn from(r: Regs) -> u8 {
        r as u8
    }
}

const ADDRESS: u8 = 0x41;

pub struct Driver<I2C> {
    i2c: I2C,
    out: u16,
}

impl<I2C> Driver<I2C> {
    pub fn new(i2c: I2C) -> Self {
        Self { i2c, out: 0xffff }
    }
}

impl<I2C: crate::I2cBus> crate::PortDriver for Driver<I2C> {
    type Error = I2C::BusError;

    fn set_high(&mut self, mask: u32) -> Result<(), Self::Error> {
        self.out |= mask as u16;
        if mask & 0x00FF != 0 {
            self.i2c
                .write_reg(ADDRESS, Regs::OutputPort0, (self.out & 0xFF) as u8)?;
        }
        if mask & 0xFF00 != 0 {
            self.i2c
                .write_reg(ADDRESS, Regs::OutputPort1, (self.out >> 8) as u8)?;
        }
        Ok(())
    }
    fn set_low(&mut self, mask: u32) -> Result<(), Self::Error> {
        self.out &= !mask as u16;
        if mask & 0x00FF != 0 {
            self.i2c
                .write_reg(ADDRESS, Regs::OutputPort0, (self.out & 0xFF) as u8)?;
        }
        if mask & 0xFF00 != 0 {
            self.i2c
                .write_reg(ADDRESS, Regs::OutputPort1, (self.out >> 8) as u8)?;
        }
        Ok(())
    }
    fn is_set_high(&mut self, mask: u32) -> Result<bool, Self::Error> {
        Ok(self.out & mask as u16 != 0)
    }
    fn is_set_low(&mut self, mask: u32) -> Result<bool, Self::Error> {
        Ok(self.out & mask as u16 == 0)
    }

    fn is_high(&mut self, mask: u32) -> Result<bool, Self::Error> {
        let io0 = if mask & 0x00FF != 0 {
            self.i2c.read_reg(ADDRESS, Regs::InputPort0)?
        } else {
            0
        };
        let io1 = if mask & 0xFF00 != 0 {
            self.i2c.read_reg(ADDRESS, Regs::InputPort1)?
        } else {
            0
        };
        let in_ = ((io1 as u16) << 8) | io0 as u16;
        Ok(in_ & mask as u16 != 0)
    }
    fn is_low(&mut self, mask: u32) -> Result<bool, Self::Error> {
        self.is_high(mask).map(|b| !b)
    }

    fn set_direction(&mut self, mask: u32, dir: crate::Direction) -> Result<(), Self::Error> {
        let (mask_set, mask_clear) = match dir {
            crate::Direction::Input => (mask as u16, 0),
            crate::Direction::Output => (0, mask as u16),
        };
        if mask & 0x00FF != 0 {
            self.i2c.update_reg(
                ADDRESS,
                Regs::Configuration0,
                (mask_set & 0xFF) as u8,
                (mask_clear & 0xFF) as u8,
            )?;
        }
        if mask & 0xFF00 != 0 {
            self.i2c.update_reg(
                ADDRESS,
                Regs::Configuration1,
                (mask_set >> 8) as u8,
                (mask_clear >> 8) as u8,
            )?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use embedded_hal_mock::i2c as mock_i2c;

    #[test]
    fn pca9555() {
        let expectations = [
            // pin setup io0_0
            mock_i2c::Transaction::write_read(super::ADDRESS, vec![0x06], vec![0xff]),
            mock_i2c::Transaction::write(super::ADDRESS, vec![0x06, 0xfe]),
            // pin setup io0_7
            mock_i2c::Transaction::write_read(super::ADDRESS, vec![0x06], vec![0xfe]),
            mock_i2c::Transaction::write(super::ADDRESS, vec![0x06, 0x7e]),
            mock_i2c::Transaction::write_read(super::ADDRESS, vec![0x06], vec![0x7e]),
            mock_i2c::Transaction::write(super::ADDRESS, vec![0x06, 0xfe]),
            // pin setup io1_0
            mock_i2c::Transaction::write_read(super::ADDRESS, vec![0x07], vec![0xff]),
            mock_i2c::Transaction::write(super::ADDRESS, vec![0x07, 0xfe]),
            // pin setup io1_7
            mock_i2c::Transaction::write_read(super::ADDRESS, vec![0x07], vec![0xfe]),
            mock_i2c::Transaction::write(super::ADDRESS, vec![0x07, 0x7e]),
            mock_i2c::Transaction::write_read(super::ADDRESS, vec![0x07], vec![0x7e]),
            mock_i2c::Transaction::write(super::ADDRESS, vec![0x07, 0xfe]),
            // output io0_0, io1_0
            mock_i2c::Transaction::write(super::ADDRESS, vec![0x02, 0xff]),
            mock_i2c::Transaction::write(super::ADDRESS, vec![0x02, 0xfe]),
            mock_i2c::Transaction::write(super::ADDRESS, vec![0x03, 0xff]),
            mock_i2c::Transaction::write(super::ADDRESS, vec![0x03, 0xfe]),
            // input io0_7, io1_7
            mock_i2c::Transaction::write_read(super::ADDRESS, vec![0x00], vec![0x80]),
            mock_i2c::Transaction::write_read(super::ADDRESS, vec![0x00], vec![0x7f]),
            mock_i2c::Transaction::write_read(super::ADDRESS, vec![0x01], vec![0x80]),
            mock_i2c::Transaction::write_read(super::ADDRESS, vec![0x01], vec![0x7f]),
        ];
        let mut bus = mock_i2c::Mock::new(&expectations);

        let mut pca = super::Pca9555::new(bus.clone());
        let pca_pins = pca.split();

        let io0_0 = pca_pins.io0_0.into_output().unwrap();
        let io0_7 = pca_pins.io0_7.into_output().unwrap();
        let io0_7 = io0_7.into_input().unwrap();

        let io1_0 = pca_pins.io1_0.into_output().unwrap();
        let io1_7 = pca_pins.io1_7.into_output().unwrap();
        let io1_7 = io1_7.into_input().unwrap();

        // output high and low
        io0_0.set_high().unwrap();
        io0_0.set_low().unwrap();
        io1_0.set_high().unwrap();
        io1_0.set_low().unwrap();

        // input high and low
        assert!(io0_7.is_high().unwrap());
        assert!(io0_7.is_low().unwrap());
        assert!(io1_7.is_high().unwrap());
        assert!(io1_7.is_low().unwrap());

        bus.done();
    }
}