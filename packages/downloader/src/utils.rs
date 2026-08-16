use quirks_path::{Path, PathToUrlError};

pub fn path_equals_as_file_url<A: AsRef<Path>, B: AsRef<Path>>(a: A, b: B) -> Result<bool, PathToUrlError> {
  let u1 = a.as_ref().to_file_url()?;
  let u2 = b.as_ref().to_file_url()?;

  Ok(u1.as_str() == u2.as_str())
}
