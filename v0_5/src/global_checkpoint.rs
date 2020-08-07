use elrond_wasm::elrond_codec::*;
use elrond_wasm::BigUintApi;
use user_fund_storage::types::*;

#[derive(Clone)]
#[derive(PartialEq)]
pub struct GlobalCheckpoint<BigUint:BigUintApi> {
    pub total_delegation_cap: BigUint,
    pub sum_unclaimed:        BigUint,
    pub total_to_swap:        BigUint,
    pub last_id:              usize,
}

impl<BigUint:BigUintApi> Encode for GlobalCheckpoint<BigUint> {
    #[inline]
    fn dep_encode_to<O: Output>(&self, dest: &mut O)  -> Result<(), EncodeError> {
        self.total_delegation_cap.dep_encode_to(dest)?;
        self.sum_unclaimed.dep_encode_to(dest)?;
        self.total_to_swap.dep_encode_to(dest)?;
        self.last_id.dep_encode_to(dest)?;
        Ok(())
    }
}

impl<BigUint:BigUintApi> Decode for GlobalCheckpoint<BigUint> {
    #[inline]
    fn dep_decode<I: Input>(input: &mut I) -> Result<Self, DecodeError> {
        Ok(GlobalCheckpoint{
            total_delegation_cap: BigUint::dep_decode(input)?,
            sum_unclaimed:        BigUint::dep_decode(input)?,
            total_to_swap:        BigUint::dep_decode(input)?,
            last_id:              usize::dep_decode(input)?,
        })
    }
}

#[derive(Clone)]
#[derive(PartialEq)]
pub struct SwapCheckpoint<BigUint:BigUintApi> {
    pub initial:   BigUint,
    pub remaining: BigUint,
    pub f_type: FundType,
}

impl<BigUint:BigUintApi> Encode for SwapCheckpoint<BigUint> {
    #[inline]
    fn dep_encode_to<O: Output>(&self, dest: &mut O)  -> Result<(), EncodeError> {
        self.initial.dep_encode_to(dest)?;
        self.remaining.dep_encode_to(dest)?;
        self.f_type.dep_encode_to(dest)?;
        Ok(())
    }
}

impl<BigUint:BigUintApi> Decode for SwapCheckpoint<BigUint> {
    #[inline]
    fn dep_decode<I: Input>(input: &mut I) -> Result<Self, DecodeError> {
        Ok(SwapCheckpoint{
            initial:   BigUint::dep_decode(input)?,
            remaining: BigUint::dep_decode(input)?,
            f_type:    FundType::dep_decode(input)?,
        })
    }
}