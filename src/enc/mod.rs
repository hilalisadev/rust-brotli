pub mod fast_log;
pub mod command;
pub mod block_split;
pub mod brotli_bit_stream;
pub mod cluster;
pub mod constants;
pub mod entropy_encode;
pub mod utf8_util;
pub mod static_dict;
pub mod static_dict_lut;
pub mod dictionary_hash;
pub mod bit_cost;
pub mod util;
pub mod literal_cost;
pub mod histogram;
pub mod backward_references;
pub mod block_splitter;
pub mod metablock;
pub mod compress_fragment_two_pass;
pub mod compress_fragment;
pub mod encode;
pub mod reader;
mod test;

use self::encode::{BrotliEncoderCreateInstance, BrotliEncoderDestroyInstance,
                   BrotliEncoderParameter, BrotliEncoderSetParameter,
                   BrotliEncoderOperation,
                   BrotliEncoderCompressStream, BrotliEncoderIsFinished};
use self::cluster::{HistogramPair};
use self::histogram::{ContextType, HistogramLiteral, HistogramCommand, HistogramDistance};
use self::command::{Command};
use self::entropy_encode::{HuffmanTree};
use super::io_wrappers::{CustomRead, CustomWrite};

#[cfg(not(feature="no-stdlib"))]
use std::io::{Read,Write, Error, ErrorKind};
#[cfg(not(feature="no-stdlib"))]
use std::io;
#[cfg(not(feature="no-stdlib"))]
pub use alloc::HeapAlloc;
#[cfg(all(feature="unsafe",not(feature="no-stdlib")))]
pub use alloc::HeapAllocUninitialized;

pub use alloc::{AllocatedStackMemory, Allocator, SliceWrapper, SliceWrapperMut, StackAllocator};

#[cfg(not(feature="no-stdlib"))]
pub use super::io_wrappers::{IntoIoReader, IoReaderWrapper, IoWriterWrapper};

#[cfg(not(any(feature="unsafe", feature="no-stdlib")))]
pub fn BrotliCompress<InputType, OutputType>(r: &mut InputType,
                                             w: &mut OutputType,
                                             quality: u32,
                                             lgwin: u32)
                                               -> Result<usize, io::Error>
  where InputType: Read,
        OutputType: Write
{
  let mut input_buffer: [u8; 4096] = [0; 4096];
  let mut output_buffer: [u8; 4096] = [0; 4096];
  BrotliCompressCustomAlloc(r,
                            w,
                            &mut input_buffer[..],
                            &mut output_buffer[..],
                            quality,
                            lgwin,
                            HeapAlloc::<u8> { default_value: 0 },
                            HeapAlloc::<u16> { default_value: 0 },
                            HeapAlloc::<i32> { default_value: 0 },
                            HeapAlloc::<u32> { default_value: 0 },
                            HeapAlloc::<Command> {
                                default_value: Command::default(),
                            },
                            HeapAlloc::<f64> { default_value: 0.0f64 },
                            HeapAlloc::<HistogramLiteral>{
                                default_value: HistogramLiteral::default(),
                            },
                            HeapAlloc::<HistogramCommand>{
                                default_value: HistogramCommand::default(),
                            },
                            HeapAlloc::<HistogramDistance>{
                                default_value: HistogramDistance::default(),
                            },
                            HeapAlloc::<HistogramPair>{
                                default_value: HistogramPair::default(),
                            },
                            HeapAlloc::<ContextType>{
                                default_value: ContextType::default(),
                            },
                            HeapAlloc::<HuffmanTree>{
                                default_value: HuffmanTree::default(),
                            })
}

#[cfg(all(feature="unsafe",not(feature="no-stdlib")))]
pub fn BrotliCompress<InputType, OutputType>(r: &mut InputType,
                                               w: &mut OutputType)
                                               -> Result<usize, io::Error>
  where InputType: Read,
        OutputType: Write
{
  let mut input_buffer: [u8; 4096] = [0; 4096];
  let mut output_buffer: [u8; 4096] = [0; 4096];
  BrotliCompressCustomAlloc(r,
                            w,
                            &mut input_buffer[..],
                            &mut output_buffer[..],
                            quality,
                            lgwin,
                            unsafe { HeapAllocUninitialized::<u8>::new() },
                            unsafe { HeapAllocUninitialized::<u16>::new() },
                            unsafe { HeapAllocUninitialized::<i32>::new() },
                            unsafe { HeapAllocUninitialized::<u32>::new() },
                            unsafe { HeapAllocUninitialized::<Command>::new() },
                            unsafe { HeapAllocUninitialized::<f64>::new() },
                            unsafe { HeapAllocUninitialized::<HistogramLiteral>::new() },
                            unsafe { HeapAllocUninitialized::<HistogramCommand>::new() },
                            unsafe { HeapAllocUninitialized::<HistogramDistance>::new() },
                            unsafe { HeapAllocUninitialized::<HistogramPair>::new() },
                            unsafe { HeapAllocUninitialized::<ContextType>::new() },
                            unsafe { HeapAllocUninitialized::<HuffmanTree>::new() },
  )
}
#[cfg(not(feature="no-stdlib"))]
pub fn BrotliCompressCustomAlloc<InputType,
                                 OutputType,
                                 AllocU8: Allocator<u8>,
                                 AllocU16: Allocator<u16>,
                                 AllocI32: Allocator<i32>,
                                 AllocU32: Allocator<u32>,
                                 AllocCommand: Allocator<Command>,
                                 AllocF64: Allocator<f64>,
                                 AllocHL: Allocator<HistogramLiteral>,
                                 AllocHC: Allocator<HistogramCommand>,
                                 AllocHD: Allocator<HistogramDistance>,
                                 AllocHP: Allocator<HistogramPair>,
                                 AllocCT: Allocator<ContextType>,
                                 AllocHT: Allocator<HuffmanTree>>
  (r: &mut InputType,
   mut w: &mut OutputType,
   input_buffer: &mut [u8],
   output_buffer: &mut [u8],
   quality: u32,
   lgwin: u32,
   alloc_u8: AllocU8,
   alloc_u16: AllocU16,
   alloc_i32: AllocI32,
   alloc_u32: AllocU32,
   alloc_mc: AllocCommand,
   alloc_f64: AllocF64,
   alloc_hl: AllocHL,
   alloc_hc: AllocHC,
   alloc_hd: AllocHD,
   alloc_hp: AllocHP,
   alloc_ct: AllocCT,
   alloc_ht: AllocHT)
   -> Result<usize, io::Error>
  where InputType: Read,
        OutputType: Write
{
  BrotliCompressCustomIo(&mut IoReaderWrapper::<InputType>(r),
                           &mut IoWriterWrapper::<OutputType>(w),
                           input_buffer,
                           output_buffer,
                           quality,
                           lgwin,
                           alloc_u8,
                           alloc_u16,
                           alloc_i32,
                           alloc_u32,
                           alloc_mc,
                           alloc_f64,
                           alloc_hl,
                           alloc_hc,
                           alloc_hd,
                           alloc_hp,
                           alloc_ct,
                           alloc_ht,
                           Error::new(ErrorKind::UnexpectedEof, "Unexpected EOF"))
}

pub fn BrotliCompressCustomIo<ErrType,
                              InputType,
                              OutputType,
                              AllocU8: Allocator<u8>,
                              AllocU16: Allocator<u16>,
                              AllocI32: Allocator<i32>,
                              AllocU32: Allocator<u32>,
                              AllocCommand: Allocator<Command>,
                              AllocF64: Allocator<f64>,
                              AllocHL: Allocator<HistogramLiteral>,
                              AllocHC: Allocator<HistogramCommand>,
                              AllocHD: Allocator<HistogramDistance>,
                              AllocHP: Allocator<HistogramPair>,
                              AllocCT: Allocator<ContextType>,
                              AllocHT: Allocator<HuffmanTree>>
  (r: &mut InputType,
   mut w: &mut OutputType,
   input_buffer: &mut [u8],
   output_buffer: &mut [u8],
   quality: u32,
   lgwin: u32,
   mu8: AllocU8,
   mu16: AllocU16,
   mi32: AllocI32,
   mu32: AllocU32,
   mc: AllocCommand,
   mut mf64: AllocF64,
   mut mhl: AllocHL,
   mut mhc: AllocHC,
   mut mhd: AllocHD,
   mut mhp: AllocHP,
   mut mct: AllocCT,
   mut mht: AllocHT,
   unexpected_eof_error_constant: ErrType)
   -> Result<usize, ErrType>
  where InputType: CustomRead<ErrType>,
        OutputType: CustomWrite<ErrType>
{
  let mut s_orig = BrotliEncoderCreateInstance(mu8, mu16, mi32, mu32, mc);
  let mut next_in_offset: usize = 0;  
  let mut next_out_offset: usize = 0;
  let mut total_out = Some(0usize);
  {
      let mut s = &mut s_orig;
      
      BrotliEncoderSetParameter(s,
                                BrotliEncoderParameter::BROTLI_PARAM_QUALITY,
                                quality as (u32));
      BrotliEncoderSetParameter(s,
                                BrotliEncoderParameter::BROTLI_PARAM_LGWIN,
                                lgwin as (u32));
      BrotliEncoderSetParameter(s, BrotliEncoderParameter::BROTLI_PARAM_MODE, 0 as (u32)); // gen, text, font
      //BrotliEncoderSetParameter(s,
      //                          BrotliEncoderParameter::BROTLI_PARAM_SIZE_HINT,
      //                          input.len() as (u32));
      let mut available_in: usize = 0;
      let mut available_out: usize = output_buffer.len();
      let mut eof = false;
      loop {
          if available_in == 0 && !eof {
              next_in_offset = 0;
              match r.read(input_buffer) {
                  Err(_) => {
                      available_in = 0;
                      eof = true;
                  },
                  Ok(size) => {
                      if size == 0 {
                          eof = true;
                      }
                      available_in = size;
                  }
              }
          }
          let op : BrotliEncoderOperation;
          if available_in == 0 {
              op = BrotliEncoderOperation::BROTLI_OPERATION_FINISH;
          } else {
              op = BrotliEncoderOperation::BROTLI_OPERATION_PROCESS;
          }
          let result = BrotliEncoderCompressStream(s,
                                                   &mut mf64, &mut mhl, &mut mhc, &mut mhd, &mut mhp, &mut mct, &mut mht,
                                                   op,
                                                   &mut available_in,
                                                   input_buffer,
                                                   &mut next_in_offset,  
                                                   &mut available_out,
                                                   output_buffer,
                                                   &mut next_out_offset,
                                                   &mut total_out);
          let fin = BrotliEncoderIsFinished(s);
          if available_out == 0 || fin != 0 {
              let lim = output_buffer.len() - available_out;
              assert_eq!(next_out_offset, lim);
              next_out_offset = 0;
              while next_out_offset < lim {
                  match w.write(&mut output_buffer[next_out_offset..lim]) {
                      Err(e) => return Err(e),
                      Ok(size) => {
                          next_out_offset += size;
                      }
                  }
              }
              available_out = output_buffer.len();
              next_out_offset = 0;
          }
          if result <= 0 {
              return Err(unexpected_eof_error_constant);
          }
          if fin != 0 {
              break;
          }
      }
      BrotliEncoderDestroyInstance(s);
  }
  Ok(total_out.unwrap())
}
