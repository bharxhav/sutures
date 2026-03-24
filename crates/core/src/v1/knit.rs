// Knit (streaming-layer) implementation for v1::Suture.
//
// Current implementation delegates to Stitch for correctness.
// A true streaming implementation (skipping intermediate Value allocation)
// can replace this later.

use super::suture::Suture;
use crate::error::Error;
use crate::knit::Knit;
use crate::seam::Seam;
use crate::stitch::Stitch;

impl Knit for Suture {
    fn knit<T: Seam + serde::Serialize>(&self, input: &T) -> Result<Vec<u8>, Error> {
        let value = self.stitch(input)?;
        serde_json::to_vec(&value).map_err(Error::Parse)
    }

    fn unknit<T: Seam + serde::de::DeserializeOwned>(&self, input: &[u8]) -> Result<T, Error> {
        let value: serde_json::Value = serde_json::from_slice(input)?;
        self.unstitch(&value)
    }
}
