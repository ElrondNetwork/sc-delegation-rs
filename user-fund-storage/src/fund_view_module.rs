imports!();

use crate::types::fund_type::*;

use crate::fund_module::*;
use crate::user_data::*;

/// Storing total stake per type the same way as we store it for users, but with user_id 0.
/// There can be no user with id 0, so the value is safe to use.
/// These values are redundant. They help avoid having to recompute the sum, especially when computing rewards.
/// At all times the values stored here must be the sums of all user values for the respective stake state,
/// no operation may break this invariant!
pub const USER_STAKE_TOTALS_ID: usize = 0;

#[elrond_wasm_derive::module(FundViewModuleImpl)]
pub trait FundViewModule {
    #[module(FundModuleImpl)]
    fn fund_module(&self) -> FundModuleImpl<T, BigInt, BigUint>;

    #[module(UserDataModuleImpl)]
    fn user_data(&self) -> UserDataModuleImpl<T, BigInt, BigUint>;

    // UTILS

    fn get_user_stake_of_type(&self, user_id: usize, fund_type: FundType) -> BigUint {
        if user_id == USER_STAKE_TOTALS_ID {
            let type_list = self.fund_module().get_fund_list_by_type(fund_type);
            type_list.total_balance
        } else {
            let user_list = self.fund_module().get_fund_list_by_user(user_id, fund_type);
            user_list.total_balance
        }
    }

    fn get_user_total_stake(&self, user_id: usize) -> BigUint {
        let mut sum = BigUint::zero();
        sum += self.get_user_stake_of_type(user_id, FundType::WithdrawOnly);
        sum += self.get_user_stake_of_type(user_id, FundType::Waiting);
        sum += self.get_user_stake_of_type(user_id, FundType::Active);
        sum += self.get_user_stake_of_type(user_id, FundType::UnStaked);
        sum += self.get_user_stake_of_type(user_id, FundType::DeferredPayment);
        sum
    }

    // GRAND TOTAL

    #[view(totalStake)]
    fn get_total_stake(&self) -> BigUint {
        self.get_user_total_stake(USER_STAKE_TOTALS_ID)
    }

    /// Yields how much a user has staked in the contract.
    #[view(getUserStake)]
    fn get_user_total_stake_endpoint(&self, user_address: Address) -> BigUint {
        let user_id = self.user_data().get_user_id(&user_address);
        if user_id == 0 {
            BigUint::zero()
        } else {
            self.get_user_total_stake(user_id)
        }
    }

    // PER USER+TYPE

    fn get_user_stake_of_type_by_address(
        &self,
        user_address: Address,
        fund_type: FundType,
    ) -> BigUint {
        let user_id = self.user_data().get_user_id(&user_address);
        if user_id == 0 {
            BigUint::zero()
        } else {
            self.get_user_stake_of_type(user_id, fund_type)
        }
    }

    #[view(getUserWithdrawOnlyStake)]
    fn get_user_withdraw_only_stake(&self, user_address: Address) -> BigUint {
        self.get_user_stake_of_type_by_address(user_address, FundType::WithdrawOnly)
    }

    #[view(getUserWaitingStake)]
    fn get_user_waiting_stake(&self, user_address: Address) -> BigUint {
        self.get_user_stake_of_type_by_address(user_address, FundType::Waiting)
    }

    #[view(getUserActiveStake)]
    fn get_user_active_stake(&self, user_address: Address) -> BigUint {
        self.get_user_stake_of_type_by_address(user_address, FundType::Active)
    }

    #[view(getUserUnstakedStake)]
    fn get_user_unstaked_stake(&self, user_address: Address) -> BigUint {
        self.get_user_stake_of_type_by_address(user_address, FundType::UnStaked)
    }

    #[view(getUserDeferredPaymentStake)]
    fn get_user_deferred_payment_stake(&self, user_address: Address) -> BigUint {
        self.get_user_stake_of_type_by_address(user_address, FundType::DeferredPayment)
    }

    // TOTAL PER TYPE

    #[view(getTotalWithdrawOnlyStake)]
    fn get_total_withdraw_only_stake(&self) -> BigUint {
        self.get_user_stake_of_type(USER_STAKE_TOTALS_ID, FundType::WithdrawOnly)
    }

    #[view(getTotalWaitingStake)]
    fn get_total_waiting_stake(&self) -> BigUint {
        self.get_user_stake_of_type(USER_STAKE_TOTALS_ID, FundType::Waiting)
    }

    #[view(getTotalActiveStake)]
    fn get_total_active_stake(&self) -> BigUint {
        self.get_user_stake_of_type(USER_STAKE_TOTALS_ID, FundType::Active)
    }

    #[view(getTotalUnstakedStake)]
    fn get_total_unstaked_stake(&self) -> BigUint {
        self.get_user_stake_of_type(USER_STAKE_TOTALS_ID, FundType::UnStaked)
    }

    #[view(getTotalDeferredPaymentStake)]
    fn get_total_deferred_payment_stake(&self) -> BigUint {
        self.get_user_stake_of_type(USER_STAKE_TOTALS_ID, FundType::DeferredPayment)
    }

    // BREAKDOWN BY TYPE

    fn get_user_stake_by_type(
        &self,
        user_id: usize,
    ) -> MultiResult5<BigUint, BigUint, BigUint, BigUint, BigUint> {
        (
            self.get_user_stake_of_type(user_id, FundType::WithdrawOnly),
            self.get_user_stake_of_type(user_id, FundType::Waiting),
            self.get_user_stake_of_type(user_id, FundType::Active),
            self.get_user_stake_of_type(user_id, FundType::UnStaked),
            self.get_user_stake_of_type(user_id, FundType::DeferredPayment),
        )
            .into()
    }

    #[view(getUserStakeByType)]
    fn get_user_stake_by_type_endpoint(
        &self,
        user_address: &Address,
    ) -> MultiResult5<BigUint, BigUint, BigUint, BigUint, BigUint> {
        let user_id = self.user_data().get_user_id(&user_address);
        if user_id == 0 {
            (
                BigUint::zero(),
                BigUint::zero(),
                BigUint::zero(),
                BigUint::zero(),
                BigUint::zero(),
            )
                .into()
        } else {
            self.get_user_stake_by_type(user_id)
        }
    }

    #[view(getTotalStakeByType)]
    fn get_total_stake_by_type_endpoint(
        &self,
    ) -> MultiResult5<BigUint, BigUint, BigUint, BigUint, BigUint> {
        self.get_user_stake_by_type(USER_STAKE_TOTALS_ID)
    }

    // DEFERRED PAYMENT BREAKDOWN

    #[view(getUserDeferredPaymentList)]
    fn get_user_deferred_payment_list(
        &self,
        user_address: &Address,
    ) -> MultiResultVec<MultiResult2<BigUint, u64>> {
        let mut result = Vec::<MultiResult2<BigUint, u64>>::new();
        let user_id = self.user_data().get_user_id(&user_address);
        if user_id > 0 {
            let _ = self.fund_module().foreach_fund_by_user_type(
                user_id,
                FundType::DeferredPayment,
                SwapDirection::Forwards,
                |fund_item| {
                    if let FundDescription::DeferredPayment { created } = fund_item.fund_desc {
                        result.push(MultiResult2::from((fund_item.balance, created)));
                    }
                },
            );
        }
        result.into()
    }

    // DEFERRED PAYMENT UTIL

    fn eligible_deferred_payment(&self, user_id: usize, n_blocks_before_claim: u64) -> BigUint {
        let current_bl_nonce = self.get_block_nonce();
        self.fund_module().query_sum_funds_by_user_type(
            user_id,
            FundType::DeferredPayment,
            |fund_desc| {
                if let FundDescription::DeferredPayment { created } = fund_desc {
                    current_bl_nonce >= created + n_blocks_before_claim
                } else {
                    false
                }
            },
        )
    }

    // FULL WAITING LIST

    #[view(getFullWaitingList)]
    fn get_full_waiting_list(&self) -> MultiResultVec<MultiResult3<Address, BigUint, u64>> {
        let mut result = Vec::<MultiResult3<Address, BigUint, u64>>::new();
        let _ = self.fund_module().foreach_fund_by_type(
            FundType::Waiting,
            SwapDirection::Forwards,
            |fund_item| {
                if let FundDescription::Waiting { created } = fund_item.fund_desc {
                    let user_address = self.user_data().get_user_address(fund_item.user_id);
                    result.push(MultiResult3::from((
                        user_address,
                        fund_item.balance,
                        created,
                    )));
                }
            },
        );
        result.into()
    }

    // FULL ACTIVE LIST

    #[view(getFullActiveList)]
    fn get_full_active_list(&self) -> MultiResultVec<MultiResult2<Address, BigUint>> {
        let mut result = Vec::<MultiResult2<Address, BigUint>>::new();
        let _ = self.fund_module().foreach_fund_by_type(
            FundType::Active,
            SwapDirection::Forwards,
            |fund_item| {
                if let FundDescription::Active = fund_item.fund_desc {
                    if !self.user_data().is_empty_user_address(fund_item.user_id) {
                        let user_address = self.user_data().get_user_address(fund_item.user_id);
                        result.push(MultiResult2::from((
                            user_address,
                            fund_item.balance,
                        )));
                    }
                }
            },
        );
        result.into()
    }
}
