pub fn decode(bytes: &[u8]) -> image::RgbaImage {
  use jxl::api::{JxlDecoder, JxlDecoderOptions, JxlOutputBuffer, JxlPixelFormat, ProcessingResult};
  fn complete<T, U>(r: ProcessingResult<T, U>) -> T {
    match r {
      ProcessingResult::Complete { result } => result,
      _ => panic!("Complete JXL file requires more input"),
    }
  }
  let mut input = bytes;
  let mut decoder = complete(JxlDecoder::new(JxlDecoderOptions::default()).process(&mut input, None).unwrap());
  let (w, h) = decoder.basic_info().size;
  decoder
    .set_pixel_format(JxlPixelFormat::rgba8(decoder.basic_info().extra_channels.len()))
    .unwrap();
  let decoder = complete(decoder.process(&mut input, None).unwrap());
  let mut pixels = vec![0; w * h * 4];
  complete(decoder.process(&mut input, &mut [JxlOutputBuffer::new(&mut pixels, h, w * 4)], None).unwrap());
  image::RgbaImage::from_raw(w as u32, h as u32, pixels).unwrap()
}
