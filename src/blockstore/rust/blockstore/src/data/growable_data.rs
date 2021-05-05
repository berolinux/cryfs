use anyhow::{ensure, Error, Result};
use std::convert::TryFrom;
use std::ops::{Deref, DerefMut};

use super::data::Data;

/// This is similar to [Data], but it remembers at compile time (in const generic arguments)
/// how much prefix bytes and suffix bytes are available. This means [GrowableData::grow_region]
/// will know at compile time if it succeeds and this can be used to write safe APIs that require
/// data types with a certain number of prefix or suffix bytes and will check that invariant at compile time.
pub struct GrowableData<const PREFIX_BYTES: usize, const SUFFIX_BYTES: usize> {
    data: Data,
}

impl<const PREFIX_BYTES: usize, const SUFFIX_BYTES: usize>
    GrowableData<PREFIX_BYTES, SUFFIX_BYTES>
{
    const PREFIX_BYTES: usize = PREFIX_BYTES;
    const SUFFIX_BYTES: usize = SUFFIX_BYTES;

    pub const fn available_prefix_bytes(&self) -> usize {
        Self::PREFIX_BYTES
    }

    pub const fn available_suffix_bytes(&self) -> usize {
        Self::SUFFIX_BYTES
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    fn _check_invariant(&self) {
        assert!(self.data.available_prefix_bytes() >= PREFIX_BYTES);
        assert!(self.data.available_suffix_bytes() >= SUFFIX_BYTES);
    }

    pub fn into_subregion<
        const DELETE_NUM_BYTES_AT_BEGINNING: usize,
        const DELETE_NUM_BYTES_AT_END: usize,
    >(
        self,
    ) -> GrowableData<
        { PREFIX_BYTES + DELETE_NUM_BYTES_AT_BEGINNING },
        { SUFFIX_BYTES + DELETE_NUM_BYTES_AT_END },
    > {
        let len = self.data.len();
        assert!(
            DELETE_NUM_BYTES_AT_BEGINNING + DELETE_NUM_BYTES_AT_END <= len,
            "Tried to delete {} + {} bytes from a data region with size {}",
            DELETE_NUM_BYTES_AT_BEGINNING,
            DELETE_NUM_BYTES_AT_END,
            len,
        );
        let result = GrowableData {
            data: self
                .data
                .into_subregion(DELETE_NUM_BYTES_AT_BEGINNING..(len - DELETE_NUM_BYTES_AT_END)),
        };
        result._check_invariant();
        result
    }

    // TODO Test
    pub fn grow_region<
        const ADD_NUM_BYTES_AT_BEGINNING: usize,
        const ADD_NUM_BYTES_AT_END: usize,
    >(
        self,
    ) -> GrowableData<
        { PREFIX_BYTES - ADD_NUM_BYTES_AT_BEGINNING },
        { SUFFIX_BYTES - ADD_NUM_BYTES_AT_END },
    > {
        // const INVARIANT: bool =
        //     GreaterEquals::<{ PREFIX_BYTES }, { ADD_NUM_BYTES_AT_BEGINNING }>::RESULT;
        // static_assertions::const_assert!({ Self::PREFIX_BYTES >= ADD_NUM_BYTES_AT_BEGINNING });
        // static_assertions::const_assert!(Self::SUFFIX_BYTES >= ADD_NUM_BYTES_AT_END);
        let result = GrowableData {
            data: self
                .data
                .grow_region(ADD_NUM_BYTES_AT_BEGINNING, ADD_NUM_BYTES_AT_END)
                .expect("Can't happen since we have static assertions against this above"),
        };
        result._check_invariant();
        result
    }

    // TODO Remove
    pub fn extract(self) -> Data {
        self.data
    }
}

impl<const PREFIX_BYTES: usize, const SUFFIX_BYTES: usize> AsRef<[u8]>
    for GrowableData<PREFIX_BYTES, SUFFIX_BYTES>
{
    fn as_ref(&self) -> &[u8] {
        self.data.as_ref()
    }
}

impl<const PREFIX_BYTES: usize, const SUFFIX_BYTES: usize> AsMut<[u8]>
    for GrowableData<PREFIX_BYTES, SUFFIX_BYTES>
{
    fn as_mut(&mut self) -> &mut [u8] {
        self.data.as_mut()
    }
}

// TODO Test
impl<const PREFIX_BYTES: usize, const SUFFIX_BYTES: usize> Deref
    for GrowableData<PREFIX_BYTES, SUFFIX_BYTES>
{
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        self.as_ref()
    }
}

// TODO Test
impl<const PREFIX_BYTES: usize, const SUFFIX_BYTES: usize> DerefMut
    for GrowableData<PREFIX_BYTES, SUFFIX_BYTES>
{
    fn deref_mut(&mut self) -> &mut [u8] {
        self.as_mut()
    }
}

impl From<Vec<u8>> for GrowableData<0, 0> {
    // TODO Test
    fn from(data: Vec<u8>) -> GrowableData<0, 0> {
        Self { data: data.into() }
    }
}

impl<const PREFIX_BYTES: usize, const SUFFIX_BYTES: usize> TryFrom<Data>
    for GrowableData<PREFIX_BYTES, SUFFIX_BYTES>
{
    // TODO Custom error type
    type Error = Error;

    // TODO Test
    fn try_from(data: Data) -> Result<Self> {
        ensure!(data.available_prefix_bytes() == PREFIX_BYTES, "The given data object has {} prefix bytes available, but we tried to convert it into a GrowableData requiring {} prefix bytes", data.available_prefix_bytes(), PREFIX_BYTES);
        ensure!(data.available_suffix_bytes() == SUFFIX_BYTES, "The given data object has {} suffix bytes available, but we tried to convert it into a GrowableData requiring {} suffix bytes", data.available_suffix_bytes(), SUFFIX_BYTES);
        Ok(Self { data })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{rngs::StdRng, RngCore, SeedableRng};

    fn data_region(size: usize, seed: u64) -> Vec<u8> {
        let mut rng = StdRng::seed_from_u64(seed);
        let mut res = vec![0; size];
        rng.fill_bytes(&mut res);
        res
    }

    #[test]
    fn given_fullrangedata_when_callingasref() {
        let data: GrowableData<0, 0> = data_region(1024, 0).into();
        assert_eq!(data.as_ref(), &data_region(1024, 0));
    }

    #[test]
    fn given_fullrangedata_when_callingasmut() {
        let mut data: GrowableData<0, 0> = data_region(1024, 0).into();
        assert_eq!(data.as_mut(), &data_region(1024, 0));
    }

    #[test]
    fn given_fullrangedata_when_gettingavailablebytes() {
        let data: GrowableData<0, 0> = data_region(1024, 0).into();
        assert_eq!(0, data.available_prefix_bytes());
        assert_eq!(0, data.available_suffix_bytes());
    }

    #[test]
    fn given_fullsubregiondata_when_callingasref() {
        let data: GrowableData<0, 0> = data_region(1024, 0).into();
        let subdata = data.into_subregion::<0, 0>();
        assert_eq!(0, subdata.available_prefix_bytes());
        assert_eq!(0, subdata.available_suffix_bytes());

        assert_eq!(subdata.as_ref(), &data_region(1024, 0));
    }

    #[test]
    fn given_fullsubregiondata_when_callingasmut() {
        let data: GrowableData<0, 0> = data_region(1024, 0).into();
        let mut subdata = data.into_subregion::<0, 0>();
        assert_eq!(0, subdata.available_prefix_bytes());
        assert_eq!(0, subdata.available_suffix_bytes());

        assert_eq!(subdata.as_mut(), &data_region(1024, 0));
    }

    #[test]
    fn given_fullsubregiondata_when_gettingavailablebytes() {
        let data: GrowableData<0, 0> = data_region(1024, 0).into();
        let subdata = data.into_subregion::<0, 0>();
        assert_eq!(0, subdata.available_prefix_bytes());
        assert_eq!(0, subdata.available_suffix_bytes());
    }

    #[test]
    fn given_openendsubregiondata_when_callingasref() {
        let data: GrowableData<0, 0> = data_region(1024, 0).into();
        let subdata = data.into_subregion::<5, 0>();
        assert_eq!(subdata.as_ref(), &data_region(1024, 0)[5..]);
    }

    #[test]
    fn given_openendsubregiondata_when_callingasmut() {
        let data: GrowableData<0, 0> = data_region(1024, 0).into();
        let mut subdata = data.into_subregion::<5, 0>();
        assert_eq!(subdata.as_mut(), &data_region(1024, 0)[5..]);
    }

    #[test]
    fn given_openendsubregiondata_when_gettingavailablebytes() {
        let data: GrowableData<0, 0> = data_region(1024, 0).into();
        let subdata = data.into_subregion::<5, 0>();
        assert_eq!(5, subdata.available_prefix_bytes());
        assert_eq!(0, subdata.available_suffix_bytes());
    }

    #[test]
    fn given_openbeginningsubregiondata_when_callingasref() {
        let data: GrowableData<0, 0> = data_region(1024, 0).into();
        let subdata = data.into_subregion::<0, 24>();
        assert_eq!(subdata.as_ref(), &data_region(1024, 0)[..1000]);
    }

    #[test]
    fn given_openbeginningsubregiondata_when_callingasmut() {
        let data: GrowableData<0, 0> = data_region(1024, 0).into();
        let mut subdata = data.into_subregion::<0, 24>();
        assert_eq!(subdata.as_mut(), &data_region(1024, 0)[..1000]);
    }

    #[test]
    fn given_openbeginningsubregiondata_when_gettingavailablebytes() {
        let data: GrowableData<0, 0> = data_region(1024, 0).into();
        let subdata = data.into_subregion::<0, 24>();
        assert_eq!(0, subdata.available_prefix_bytes());
        assert_eq!(24, subdata.available_suffix_bytes());
    }

    #[test]
    fn given_subregiondata_when_callingasref() {
        let data: GrowableData<0, 0> = data_region(1024, 0).into();
        let subdata = data.into_subregion::<5, 24>();
        assert_eq!(subdata.as_ref(), &data_region(1024, 0)[5..1000]);
    }

    #[test]
    fn given_subregiondata_when_callingasmut() {
        let data: GrowableData<0, 0> = data_region(1024, 0).into();
        let mut subdata = data.into_subregion::<5, 24>();
        assert_eq!(subdata.as_mut(), &data_region(1024, 0)[5..1000]);
    }

    #[test]
    fn given_subregiondata_when_gettingavailablebytes() {
        let data: GrowableData<0, 0> = data_region(1024, 0).into();
        let subdata = data.into_subregion::<5, 24>();
        assert_eq!(5, subdata.available_prefix_bytes());
        assert_eq!(24, subdata.available_suffix_bytes());
    }

    #[test]
    fn nested_subregions_still_do_the_right_thing() {
        let data: GrowableData<0, 0> = data_region(1024, 0).into();
        let subdata = data
            .into_subregion::<0, 0>() // into_subregion(..) -> new length: 1024
            .into_subregion::<5, 0>() // into_subregion(5..) -> new length: 1019
            .into_subregion::<0, 19>() //into_subregion(..1000) -> new length: 1000
            .into_subregion::<0, 49>() //into_subregion(..951) -> new length: 951
            .into_subregion::<10, 51>() //into_subregion(10..900) -> new length: 890
            .into_subregion::<3, 89>() //into_subregion(3..801) -> new length: 798
            // and all types of ranges again, just in case they don't work if a certain other range happens beforehand
            .into_subregion::<0, 0>() // into_subregion(..) -> new length: 798
            .into_subregion::<5, 0>() // into_subregion(5..) -> new length: 793
            .into_subregion::<0, 93>() // into_subregion(..700) -> new length: 700
            .into_subregion::<0, 49>() // into_subregion(..651) -> new length: 651
            .into_subregion::<10, 51>() //into_subregion(10..600) -> new_length: 590
            .into_subregion::<3, 89>(); //into_subregion(3..501) -> new_length: 498
        assert_eq!(36, subdata.available_prefix_bytes());
        assert_eq!(490, subdata.available_suffix_bytes());
        assert_eq!(
            subdata.as_ref(),
            &data_region(1024, 0)[..][5..][..1000][..=950][10..900][3..=800][..][5..][..700]
                [..=650][10..600][3..=500]
        );
    }
}
