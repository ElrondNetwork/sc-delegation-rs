#![no_std]
#![allow(unused_attributes)]
#![allow(clippy::string_lit_as_bytes)]

#[cfg(feature = "delegation_v0_5_x_default")]
pub use delegation_v0_5_x_default as delegation_v0_5_x;
#[cfg(feature = "delegation_v0_5_x_wasm")]
pub use delegation_v0_5_x_wasm as delegation_v0_5_x;

use delegation_v0_5_x::*;

imports!();

#[elrond_wasm_derive::contract(DelegationImpl)]
pub trait Delegation {
    // METADATA

    #[endpoint]
    fn version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    // MODULES

    #[module(EventsModuleImpl)]
    fn events(&self) -> EventsModuleImpl<T, BigInt, BigUint>;

    #[module(NodeConfigModuleImpl)]
    fn node_config(&self) -> NodeConfigModuleImpl<T, BigInt, BigUint>;

    #[module(RewardsModuleImpl)]
    fn rewards(&self) -> RewardsModuleImpl<T, BigInt, BigUint>;

    #[module(ResetCheckpointsModuleImpl)]
    fn reset_checkpoints(&self) -> ResetCheckpointsModuleImpl<T, BigInt, BigUint>;

    #[module(SettingsModuleImpl)]
    fn settings(&self) -> SettingsModuleImpl<T, BigInt, BigUint>;

    #[module(PauseModuleImpl)]
    fn pause(&self) -> PauseModuleImpl<T, BigInt, BigUint>;

    #[module(UserStakeModuleImpl)]
    fn user_stake(&self) -> UserStakeModuleImpl<T, BigInt, BigUint>;

    #[module(NodeActivationModuleImpl)]
    fn node_activation(&self) -> NodeActivationModuleImpl<T, BigInt, BigUint>;

    #[module(UserUnStakeModuleImpl)]
    fn user_unstake(&self) -> UserUnStakeModuleImpl<T, BigInt, BigUint>;

    #[module(UserDataModuleImpl)]
    fn user_data(&self) -> UserDataModuleImpl<T, BigInt, BigUint>;

    #[module(FundTransformationsModuleImpl)]
    fn fund_transf_module(&self) -> FundTransformationsModuleImpl<T, BigInt, BigUint>;

    #[module(FundViewModuleImpl)]
    fn fund_view_module(&self) -> FundViewModuleImpl<T, BigInt, BigUint>;

    // INIT - update from genesis version

    /// extremely dangerous, should only happen once during upgrade
    fn change_user_address(&self, old_address: &Address, new_address: &Address) -> SCResult<()> {
        let user_id = self.user_data().get_user_id(old_address);
        require!(user_id > 1, "old address does not exist or is owner");
        let new_address_user_id = self.user_data().get_user_id(new_address);
        require!(new_address_user_id == 0, "new address already a delegator");
        self.user_data().set_user_id(old_address, 0);
        self.user_data().set_user_id(new_address, user_id);
        self.user_data().set_user_address(user_id, new_address);
        Ok(())
    }

    #[init]
    fn init(&self, #[var_args] change_user_addresses: VarArgs<MultiArg2<Address, Address>>) -> SCResult<()> {
        // the genesis contract didn't have the concept of total delegation cap
        // so the field needs to be updated here to correspond to how much was staked
        let total_active = self
            .fund_view_module()
            .get_user_stake_of_type(USER_STAKE_TOTALS_ID, FundType::Active);
        self.settings().set_total_delegation_cap(total_active);

        for change_user_address_arg_pair in change_user_addresses.into_vec() {
            let (old_address, new_address) = change_user_address_arg_pair.into_tuple();
            sc_try!(self.change_user_address(&old_address, &new_address));
        }

        Ok(())
    }

    // Callbacks can only be declared here for the moment.

    #[callback]
    fn auction_stake_callback(
        &self,
        #[callback_arg] node_ids: Vec<usize>,
        call_result: AsyncCallResult<VarArgs<BLSStatusMultiArg>>,
    ) {
        self.node_activation()
            .auction_stake_callback(node_ids, call_result)
            .unwrap();
        // TODO: replace unwrap with typical Result handling
    }

    #[callback]
    fn auction_unstake_callback(
        &self,
        #[callback_arg] node_ids: Vec<usize>,
        call_result: AsyncCallResult<VarArgs<BLSStatusMultiArg>>,
    ) {
        self.node_activation()
            .auction_unstake_callback(node_ids, call_result)
            .unwrap();
        // TODO: replace unwrap with typical Result handling
    }

    #[callback]
    fn auction_unbond_callback(
        &self,
        #[callback_arg] node_ids: Vec<usize>,
        call_result: AsyncCallResult<VarArgs<BLSStatusMultiArg>>,
    ) {
        self.node_activation()
            .auction_unbond_callback(node_ids, call_result)
            .unwrap();
        // TODO: replace unwrap with typical Result handling
    }
}
