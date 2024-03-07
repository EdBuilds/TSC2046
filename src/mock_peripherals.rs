use mockall::*;

#[derive(Debug, PartialEq)]
pub enum Error {}
impl embedded_hal::spi::Error for Error {
    fn kind(&self) -> embedded_hal::spi::ErrorKind {
        todo!()
    }
}

mock! {
    pub SpiBus {} // Name of the mock struct, less the "Mock" prefix
    impl embedded_hal::spi::SpiBus for SpiBus { // specification of the trait to mock
fn flush(&mut self) -> Result<(), <Self as embedded_hal::spi::ErrorType>::Error> { todo!() }
fn transfer_in_place(&mut self, a: &mut [u8]) -> Result<(), <Self as embedded_hal::spi::ErrorType>::Error> { todo!() }
fn transfer(&mut self, a: &mut [u8], b: &[u8]) -> Result<(), <Self as embedded_hal::spi::ErrorType>::Error> { todo!() }
fn write(&mut self, a: &[u8]) -> Result<(), <Self as embedded_hal::spi::ErrorType>::Error> { todo!() }
fn read(&mut self, a: &mut [u8]) -> Result<(), <Self as embedded_hal::spi::ErrorType>::Error> { todo!() }
    }
    impl embedded_hal::spi::ErrorType for SpiBus {
    type Error = Error;
}
}
mock! {
pub SimpleHalSpiDevice {
}

impl embedded_hal::spi::SpiDevice<u8> for SimpleHalSpiDevice {
    fn transaction<'a>(
        &mut self,
        operations: &mut [embedded_hal::spi::Operation<'a, u8>],
    ) -> Result<(), Error> {
        for op in operations {
            match op {
                embedded_hal::spi::Operation::Read(read) => {
                    self.bus.read(read).unwrap();
                }
                embedded_hal::spi::Operation::Write(write) => {
                    self.bus.write(write).unwrap();
                }
                embedded_hal::spi::Operation::Transfer(read, write) => {
                    self.bus.transfer(read, write).unwrap();
                }
                embedded_hal::spi::Operation::TransferInPlace(words) => {
                    self.bus.transfer_in_place(words).unwrap();
                }
                embedded_hal::spi::Operation::DelayNs(us) => {
                    //embedded_hal::delay::DelayNs::delay_us(&mut Delay::new(), *us);
                }
            }
        }
        Ok(())
    }
}

impl embedded_hal::spi::ErrorType for SimpleHalSpiDevice {
    type Error = Error;
}
    }

#[derive(Debug, PartialEq, Eq)]
pub enum MockOperation<'a, Word: 'static> {
    Read(&'a [Word]),
    Write(&'a [Word]),
}
