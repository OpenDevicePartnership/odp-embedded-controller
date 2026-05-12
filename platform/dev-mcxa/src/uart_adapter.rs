//! Adapter wrapping [`embassy_mcxa::lpuart::Lpuart`] to expose the
//! `embedded-io-async` 0.6 trait surface required by the OpenDevicePartnership
//! `uart-service` crate.
//!
//! The upstream `embassy-mcxa` HAL implements `embedded-io-async` 0.7, while
//! `uart-service` still depends on the 0.6 traits.
//!
//! TODO: once uart-service is updated to use embedded-io-async 0.7, this can be removed.

use embassy_mcxa::lpuart;

/// Type-erased UART error suitable for the 0.6 `embedded-io` trait family.
#[derive(Debug, defmt::Format)]
pub struct UartError;

impl embedded_io_6::Error for UartError {
    fn kind(&self) -> embedded_io_6::ErrorKind {
        embedded_io_6::ErrorKind::Other
    }
}

/// UART wrapper to bridge embedded-io-async v0.6 traits over an MCXA DMA LPUART.
pub struct UartAdapter(pub lpuart::LpuartBbq);

impl embedded_io_6::ErrorType for UartAdapter {
    type Error = UartError;
}

impl embedded_io_async_6::Read for UartAdapter {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.0.read(buf).await.map_err(|_| UartError)
    }
}

impl embedded_io_async_6::Write for UartAdapter {
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        self.0.write(buf).await.map_err(|_| UartError)
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        self.0.flush().await;
        Ok(())
    }
}
