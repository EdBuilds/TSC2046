#![cfg_attr(not(test), no_std)]

#[cfg(test)]
mod mock_peripherals;
mod types;

use embedded_hal::spi::{ErrorType, Operation, SpiDevice};
use types::{Axes, ControlBit};

#[derive(Debug, PartialEq)]
/// Struct representing a touch point on the touch screen.
pub struct TouchPoint {
    /// The x-coordinate of the touch point, ranging from 0 to 4096.
    pub x: u16,
    /// The y-coordinate of the touch point, ranging from 0 to 4096.
    pub y: u16,
    /// The pressure value of the touch point, ranging from 0.0 (max pressure) to the set touch threshold.
    pub z: f32,
}
/// Driver for the TSC2046 4-wire touch screen controller.
pub struct Tsc2046<SPI> {
    /// The SPI interface used to communicate with the TSC2046 chip.
    spi: SPI,
    /// Whether the interrupt pin is enabled or not.
    irq_on: bool,
    /// The minimum pressure value required to register a touch event.
    touch_threshold: f32,
}
impl<SPI> Tsc2046<SPI>
where
    SPI: SpiDevice,
{
    /// Creates a new instance of the `Tsc2046` driver.
    ///
    /// # Arguments
    ///
    /// * `spi` - The SPI interface used to communicate with the TSC2046 chip.
    /// * `irq_on` - Whether to enable the interrupt pin or not.
    /// * `touch_threshold` - The minimum pressure value required to register a touch event.
    ///
    /// # Returns
    ///
    /// A `Result` containing the `Tsc2046` instance or an error if the register update fails.
    pub fn new(
        spi: SPI,
        irq_on: bool,
        touch_threshold: f32,
    ) -> Result<Self, <SPI as ErrorType>::Error> {
        let mut instance = Self {
            spi,
            irq_on,
            touch_threshold,
        };
        instance.update_register()?;
        Ok(instance)
    }
    /// Updates the control register of the TSC2046 chip.
    ///
    /// # Returns
    ///
    /// A `Result` indicating whether the register update was successful or not.
    fn update_register(&mut self) -> Result<(), <SPI as ErrorType>::Error> {
        let mut control_word = ControlBit::S; //start bit always on
        control_word &= !ControlBit::MODE; // 12 bit mode
        control_word &= !ControlBit::SER; // enable differential mode
        control_word |= Axes::X.ctrl_bits();
        if self.irq_on {
            control_word &= !ControlBit::PD0;
            control_word &= !ControlBit::PD1;
        } else {
            control_word |= ControlBit::PD0;
            control_word |= ControlBit::PD1;
        }

        let mut buf = [0_u8; 2];
        self.spi.transaction(&mut [
            Operation::Write(&[control_word.bits()]),
            Operation::Read(&mut buf),
        ])
    }
    /// Reads the value of the specified axis from the TSC2046 chip.
    ///
    /// # Arguments
    ///
    /// * `axis` - The axis to read.
    ///
    /// # Returns
    ///
    /// A `Result` containing the raw value of the specified axis or an error if the read fails.
    fn read_axis(&mut self, axis: Axes) -> Result<u16, <SPI as ErrorType>::Error> {
        let mut control_word = ControlBit::S; //start bit always on
        control_word &= !ControlBit::MODE; // 12 bit mode
        control_word &= !ControlBit::SER; // enable differential mode
        control_word |= axis.ctrl_bits();

        if self.irq_on {
            control_word &= !ControlBit::PD0;
            control_word &= !ControlBit::PD1;
        } else {
            control_word |= ControlBit::PD0;
            control_word |= ControlBit::PD1;
        }

        let mut buf = [0_u8; 2];
        self.spi.transaction(&mut [
            Operation::Write(&[control_word.bits()]),
            Operation::Read(&mut buf),
        ])?;
        Ok((((buf[0] as u16) << 8 | buf[1] as u16) >> 3) & 0xFFF)
    }

    /// Enables or disables the interrupt pin.
    ///
    /// # Arguments
    ///
    /// * `enable_irq` - Whether to enable or disable the interrupt pin.
    ///
    /// # Returns
    ///
    /// A `Result` indicating whether the interrupt pin configuration was successful or not.
    pub fn set_irq(&mut self, enable_irq: bool) -> Result<(), <SPI as ErrorType>::Error> {
        self.irq_on = enable_irq;
        self.update_register()
    }

    /// Sets the minimum pressure value required to register a touch event.
    ///
    /// # Arguments
    ///
    /// * `touch_threshold` - The minimum pressure value (between 0.0 and inf).   
    pub fn set_touch_threshold(&mut self, touch_threshold: f32) {
        self.touch_threshold = touch_threshold;
    }

    /// Reads the touch point from the TSC2046 chip.
    ///
    /// # Returns
    ///
    /// A `Result` containing the `TouchPoint` struct if a touch event is detected, or `None` if no touch event is detected or an error occurs during the read operation.
    /// The `x` and `y` coordinates of the `TouchPoint` are in the range of 0 to 4096.
    pub fn get_touch(&mut self) -> Result<Option<TouchPoint>, <SPI as ErrorType>::Error> {
        let x_raw = self.read_axis(Axes::X)?;
        let y_raw = self.read_axis(Axes::Y)?;
        let z1_raw = self.read_axis(Axes::Z1)?;
        let z2_raw = self.read_axis(Axes::Z2)?;
        let z_value = x_raw as f32 / 4096_f32 * (z2_raw as f32 / z1_raw as f32 - 1.0f32);
        if z_value < self.touch_threshold {
            Ok(Some(TouchPoint {
                x: x_raw,
                y: y_raw,
                z: z_value,
            }))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock_peripherals::{MockOperation, MockSimpleHalSpiDevice};

    // Predefined control words for testing
    const CTRL_WORD_X_NO_IRQ: u8 = 0b11010011;
    const CTRL_WORD_Y_NO_IRQ: u8 = 0b10010011;
    const CTRL_WORD_Z1_NO_IRQ: u8 = 0b10110011;
    const CTRL_WORD_Z2_NO_IRQ: u8 = 0b11000011;

    const CTRL_WORD_X_IRQ: u8 = 0b11010000;
    const CTRL_WORD_Y_IRQ: u8 = 0b10010000;
    const CTRL_WORD_Z1_IRQ: u8 = 0b10110000;
    const CTRL_WORD_Z2_IRQ: u8 = 0b11000000;

    //Helper function to assert SPI operations
    fn assert_spi_operations<Word: std::cmp::PartialEq + std::fmt::Debug + std::marker::Copy>(
        ops: &mut [Operation<'_, Word>],
        expected_ops: &[MockOperation<'_, Word>],
    ) {
        assert_eq!(ops.len(), expected_ops.len());
        let both_ops = ops.iter_mut().zip(expected_ops.iter());
        for (op, expected_op) in both_ops {
            match expected_op {
                MockOperation::Read(expected_read_buf) => {
                    match op {
                        // In case of a reading operation, we copy the expected values into the buffer.
                        Operation::Read(op_read_buf) => {
                            assert_eq!(op_read_buf.len(), expected_read_buf.len());
                            for i in 0..op_read_buf.len() {
                                op_read_buf[i] = expected_read_buf[i];
                            }
                        }
                        _ => assert!(false),
                    }
                }
                MockOperation::Write(expected_write_buf) => {
                    match op {
                        // In case of a Writing operation, the value of the buffer is compared against the value of the expected buffer.
                        Operation::Write(op_write_buf) => {
                            assert_eq!(op_write_buf, expected_write_buf);
                        }
                        _ => assert!(false),
                    }
                }
            }
        }
    }

    static X_TOUCH_VALUE: u16 = 100;
    static Y_TOUCH_VALUE: u16 = 100;
    static Z1_TOUCH_VALUE: u16 = 5;
    static Z2_TOUCH_VALUE: u16 = 2053;

    static INIT_RETURN_BUF: [u8; 2] = [0, 0];
    static READ_X_RETURN_BUF: [u8; 2] = [(X_TOUCH_VALUE >> 5) as u8, (X_TOUCH_VALUE << 3) as u8];
    static READ_Y_RETURN_BUF: [u8; 2] = [(Y_TOUCH_VALUE >> 5) as u8, (Y_TOUCH_VALUE << 3) as u8];
    static READ_Z1_RETURN_BUF: [u8; 2] = [(Z1_TOUCH_VALUE >> 5) as u8, (Z1_TOUCH_VALUE << 3) as u8];
    static READ_Z2_RETURN_BUF: [u8; 2] = [(Z2_TOUCH_VALUE >> 5) as u8, (Z2_TOUCH_VALUE << 3) as u8];
    #[test]
    fn test_get_touch_no_irq() {
        let expected_ops_init = [
            MockOperation::Write(&[CTRL_WORD_X_NO_IRQ]),
            MockOperation::Read(&INIT_RETURN_BUF),
        ];
        let expected_ops_x = [
            MockOperation::Write(&[CTRL_WORD_X_NO_IRQ]),
            MockOperation::Read(&READ_X_RETURN_BUF),
        ];
        let expected_ops_y = [
            MockOperation::Write(&[CTRL_WORD_Y_NO_IRQ]),
            MockOperation::Read(&READ_Y_RETURN_BUF),
        ];
        let expected_ops_z1 = [
            MockOperation::Write(&[CTRL_WORD_Z1_NO_IRQ]),
            MockOperation::Read(&READ_Z1_RETURN_BUF),
        ];
        let expected_ops_z2 = [
            MockOperation::Write(&[CTRL_WORD_Z2_NO_IRQ]),
            MockOperation::Read(&READ_Z2_RETURN_BUF),
        ];
        let expected_touch_point = TouchPoint {
            x: 100,
            y: 100,
            z: 10.0f32,
        };
        let mut mock_spi_dev = MockSimpleHalSpiDevice::new();

        mock_spi_dev
            .expect_transaction()
            .times(1)
            .returning(move |operations| {
                assert_spi_operations(operations, &expected_ops_init);
                Ok(())
            });
        mock_spi_dev
            .expect_transaction()
            .times(1)
            .returning(move |operations| {
                assert_spi_operations(operations, &expected_ops_x);
                Ok(())
            });
        mock_spi_dev
            .expect_transaction()
            .times(1)
            .returning(move |operations| {
                assert_spi_operations(operations, &expected_ops_y);
                Ok(())
            });
        mock_spi_dev
            .expect_transaction()
            .times(1)
            .returning(move |operations| {
                assert_spi_operations(operations, &expected_ops_z1);
                Ok(())
            });
        mock_spi_dev
            .expect_transaction()
            .times(1)
            .returning(move |operations| {
                assert_spi_operations(operations, &expected_ops_z2);
                Ok(())
            });
        let mut test_driver =
            Tsc2046::new(mock_spi_dev, false, 100.0).expect("Could not create driver");
        assert_eq!(test_driver.get_touch(), Ok(Some(expected_touch_point)));
    }

    #[test]
    fn test_get_touch_irq() {
        let expected_ops_init = [
            MockOperation::Write(&[CTRL_WORD_X_NO_IRQ]),
            MockOperation::Read(&INIT_RETURN_BUF),
        ];
        let expected_ops_irq_set = [
            MockOperation::Write(&[CTRL_WORD_X_IRQ]),
            MockOperation::Read(&INIT_RETURN_BUF),
        ];

        let expected_ops_x = [
            MockOperation::Write(&[CTRL_WORD_X_IRQ]),
            MockOperation::Read(&READ_X_RETURN_BUF),
        ];
        let expected_ops_y = [
            MockOperation::Write(&[CTRL_WORD_Y_IRQ]),
            MockOperation::Read(&READ_Y_RETURN_BUF),
        ];
        let expected_ops_z1 = [
            MockOperation::Write(&[CTRL_WORD_Z1_IRQ]),
            MockOperation::Read(&READ_Z1_RETURN_BUF),
        ];
        let expected_ops_z2 = [
            MockOperation::Write(&[CTRL_WORD_Z2_IRQ]),
            MockOperation::Read(&READ_Z2_RETURN_BUF),
        ];
        let expected_touch_point = TouchPoint {
            x: 100,
            y: 100,
            z: 10.0f32,
        };
        let mut mock_spi_dev = MockSimpleHalSpiDevice::new();

        mock_spi_dev
            .expect_transaction()
            .times(1)
            .returning(move |operations| {
                assert_spi_operations(operations, &expected_ops_init);
                Ok(())
            });
        mock_spi_dev
            .expect_transaction()
            .times(1)
            .returning(move |operations| {
                assert_spi_operations(operations, &expected_ops_irq_set);
                Ok(())
            });
        mock_spi_dev
            .expect_transaction()
            .times(1)
            .returning(move |operations| {
                assert_spi_operations(operations, &expected_ops_x);
                Ok(())
            });
        mock_spi_dev
            .expect_transaction()
            .times(1)
            .returning(move |operations| {
                assert_spi_operations(operations, &expected_ops_y);
                Ok(())
            });
        mock_spi_dev
            .expect_transaction()
            .times(1)
            .returning(move |operations| {
                assert_spi_operations(operations, &expected_ops_z1);
                Ok(())
            });
        mock_spi_dev
            .expect_transaction()
            .times(1)
            .returning(move |operations| {
                assert_spi_operations(operations, &expected_ops_z2);
                Ok(())
            });
        let mut test_driver =
            Tsc2046::new(mock_spi_dev, false, 100.0).expect("Could not create driver");
        test_driver.set_irq(true).expect("Could not set IRQ");
        assert_eq!(test_driver.get_touch(), Ok(Some(expected_touch_point)));
    }
}
