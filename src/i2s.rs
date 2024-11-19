use esp_hal::{
    dma::{Channel, DmaChannelConvert, DmaDescriptor, ReadBuffer},
    i2s::{asynch::I2sWriteDmaTransferAsync, DataFormat, I2s, RegisterAccess, Standard},
    peripheral::Peripheral,
    prelude::*,
    Mode,
};
use static_cell::StaticCell;

pub const CHUNK_SAMPLES: usize = 256; // max samples per write
pub const SAMPLE_RATE: u32 = 41_000; // samples per second
pub const NUM_CHANNEL: usize = 2; // stereo
pub const DATA_FORMAT: DataFormat = DataFormat::Data16Channel16;

pub const BYTES_PER_SAMPLE: usize = NUM_CHANNEL * 2;
pub const DMA_NUM: usize = 6;

pub type Sample = [i16; NUM_CHANNEL];

const CHUNK_BYTES: usize = BYTES_PER_SAMPLE * CHUNK_SAMPLES;
const TX_BYTES: usize = DMA_NUM * CHUNK_BYTES;
static TX_BUFFER: StaticCell<[u8; TX_BYTES]> = StaticCell::new();
static TX_DESCRIPTORS: StaticCell<[DmaDescriptor; DMA_NUM]> = StaticCell::new();
static RX_DESCRIPTORS: StaticCell<[DmaDescriptor; 0]> = StaticCell::new();

pub fn new_i2s<'d, I, CH, DmaMode>(
    i2s: impl Peripheral<P = I> + 'd,
    dma_channel: Channel<'d, CH, DmaMode>,
) -> I2s<'d, I, DmaMode>
where
    I: RegisterAccess,
    CH: DmaChannelConvert<I::Dma>,
    DmaMode: Mode,
{
    // initialize descriptors
    // see convenience macro [dma_buffer_chunk_size!] from esp-hal for reference
    let tx_descriptors = TX_DESCRIPTORS.init([DmaDescriptor::EMPTY; DMA_NUM]);
    // for now, we have to pass rx descriptors even though we're not using an rx buffer
    let rx_descriptors = RX_DESCRIPTORS.init([DmaDescriptor::EMPTY; 0]);
    I2s::new(
        i2s,
        Standard::Philips,
        DATA_FORMAT,
        SAMPLE_RATE.Hz(),
        dma_channel,
        rx_descriptors,
        tx_descriptors,
    )
}

pub fn take_tx_buffer() -> &'static mut [u8; TX_BYTES] {
    TX_BUFFER.init([0u8; TX_BYTES])
}

pub fn new_chunk_buffer() -> [Sample; CHUNK_SAMPLES] {
    [[0; NUM_CHANNEL]; CHUNK_SAMPLES]
}

pub async fn push<'d, T, TXBUF>(
    transfer: &mut I2sWriteDmaTransferAsync<'d, T, TXBUF>,
    samples: &[Sample],
) -> usize
where
    T: RegisterAccess,
    TXBUF: ReadBuffer,
{
    let data = bytemuck::cast_slice(samples);
    let written_bytes = transfer.push(data).await.unwrap();
    written_bytes / BYTES_PER_SAMPLE
}
