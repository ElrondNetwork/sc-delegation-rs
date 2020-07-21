
use crate::fund_type::*;

use super::settings::*;
use crate::events::*;
use crate::node_config::*;
use crate::user_stake::*;
use crate::node_activation::*;
use crate::user_data::*;
use crate::fund_transf_module::*;
use crate::fund_view_module::*;

imports!();

// Groups together data per delegator from the storage.
pub struct UserRewardData<BigUint> {
    /// The value of the total cumulated rewards in the contract when the user's rewards were computed the last time.
    pub reward_checkpoint: BigUint,

    /// Rewards that are computed but not yet sent to the delegator.
    pub unclaimed_rewards: BigUint,
}

/// Contains logic to compute and distribute individual delegator rewards.
#[elrond_wasm_derive::module(RewardsModuleImpl)]
pub trait RewardsModule {

    #[module(EventsModuleImpl)]
    fn events(&self) -> EventsModuleImpl<T, BigInt, BigUint>;

    #[module(NodeConfigModuleImpl)]
    fn node_config(&self) -> NodeConfigModuleImpl<T, BigInt, BigUint>;

    #[module(SettingsModuleImpl)]
    fn settings(&self) -> SettingsModuleImpl<T, BigInt, BigUint>;

    #[module(UserDataModuleImpl)]
    fn user_data(&self) -> UserDataModuleImpl<T, BigInt, BigUint>;

    #[module(FundTransformationsModuleImpl)]
    fn fund_transf_module(&self) -> FundTransformationsModuleImpl<T, BigInt, BigUint>;

    #[module(FundViewModuleImpl)]
    fn fund_view_module(&self) -> FundViewModuleImpl<T, BigInt, BigUint>;

    #[module(UserStakeModuleImpl)]
    fn user_stake(&self) -> UserStakeModuleImpl<T, BigInt, BigUint>;

    #[module(NodeActivationModuleImpl)]
    fn node_activation(&self) -> NodeActivationModuleImpl<T, BigInt, BigUint>;

    /// Claiming rewards has 2 steps:
    /// 1. computing the delegator rewards out of the total rewards, and
    /// 2. sending those rewards to the delegator address. 
    /// This field keeps track of rewards that went through step 1 but not 2,
    /// i.e. were computed and deducted from the total rewards, but not yet "physically" sent to the user.
    /// The unclaimed stake still resides in the contract.
    #[storage_get("u_rew_unclmd")]
    fn get_user_rew_unclaimed(&self, user_id: usize) -> BigUint;

    #[storage_set("u_rew_unclmd")]
    fn set_user_rew_unclaimed(&self, user_id: usize, user_rew_unclaimed: &BigUint);

    /// As the time passes, if the contract is active, rewards periodically arrive in the contract. 
    /// Users can claim their share of rewards anytime.
    /// This field helps keeping track of how many rewards came to the contract since the last claim.
    /// More specifically, it indicates the cumulated sum of rewards that had arrived in the contract 
    /// when the user last claimed their own personal rewards.
    /// If zero, it means the user never claimed any rewards.
    /// If equal to get_total_cumulated_rewards, it means the user claimed everything there is for him/her.
    #[storage_get("u_rew_checkp")]
    fn get_user_rew_checkpoint(&self, user_id: usize) -> BigUint;

    #[storage_set("u_rew_checkp")]
    fn set_user_rew_checkpoint(&self, user_id: usize, user_rew_checkpoint: &BigUint);

    #[storage_get("sent_rewards")]
    fn get_sent_rewards(&self) -> BigUint;

    #[storage_set("sent_rewards")]
    fn set_sent_rewards(&self, sent_rewards: &BigUint);

    /// Yields all the rewards received by the contract since its creation.
    /// This value is monotonously increasing - it can never decrease.
    /// Handing out rewards will not decrease this value.
    /// This is to keep track of how many funds entered the contract. It ignores any funds leaving the contract.
    /// Individual rewards are computed based on this value.
    /// For each user we keep a record on what was the value of the historical rewards when they last claimed.
    /// Subtracting that from the current historical rewards yields how much accumulated in the contract since they last claimed.
    #[view(getTotalCumulatedRewards)]
    fn get_total_cumulated_rewards(&self) -> BigUint {
        self.storage_load_cumulated_validator_reward()
    }

    /// The account running the nodes is entitled to (service_fee / NODE_DENOMINATOR) * rewards.
    fn service_fee_reward(&self, tot_rewards: &BigUint) -> BigUint {
        let mut node_rewards = tot_rewards * &self.node_config().get_service_fee();
        node_rewards /= BigUint::from(PERCENTAGE_DENOMINATOR);
        node_rewards
    }

    /// Does not update storage, only returns the user rewards object, after computing rewards.
    fn load_updated_user_rewards(&self, user_id: usize) -> UserRewardData<BigUint> {
        let mut user_data = self.load_user_reward_data(user_id);

        // new rewards are what was added since the last time rewards were computed
        let tot_cumul_rewards = self.get_total_cumulated_rewards();
        let tot_new_rewards = &tot_cumul_rewards - &user_data.reward_checkpoint;
        if tot_new_rewards == 0 {
            return user_data; // nothing happened since the last claim
        }

        // the owner is entitled to: new rewards * service_fee / NODE_DENOMINATOR
        let service_fee = self.service_fee_reward(&tot_new_rewards);
        
        // total stake that gets rewarded
        let total_active_stake_for_sale =
            self.fund_view_module().get_user_stake_of_type(USER_STAKE_TOTALS_ID, FundType::UnStaked);
        let total_active_stake =
            &self.fund_view_module().get_user_stake_of_type(USER_STAKE_TOTALS_ID, FundType::Active) +
            &total_active_stake_for_sale;

        // update node rewards, if applicable
        if user_id == OWNER_USER_ID {
            // the owner gets the service fee
            user_data.unclaimed_rewards += &service_fee;

            // the owner also gets all rewards for all the UnStaked stake
            if total_active_stake_for_sale > 0 {
                let mut active_for_sale_new_rewards = &tot_new_rewards - &service_fee;
                active_for_sale_new_rewards *= &total_active_stake_for_sale;
                active_for_sale_new_rewards /= &total_active_stake;
                user_data.unclaimed_rewards += &active_for_sale_new_rewards;
            }
        }

        // update delegator rewards based on Active stake
        let u_stake_active = self.fund_view_module().get_user_stake_of_type(user_id, FundType::Active);
        if u_stake_active > 0 {
            // delegator reward is:
            // total new rewards * (1 - service_fee / NODE_DENOMINATOR) * user stake / total stake
            let mut delegator_new_rewards = &tot_new_rewards - &service_fee;
            delegator_new_rewards *= &u_stake_active;
            delegator_new_rewards /= &total_active_stake;
            user_data.unclaimed_rewards += &delegator_new_rewards;
        }

        // update user data checkpoint
        user_data.reward_checkpoint = tot_cumul_rewards;

        user_data
    }

    /// Convenience method, brings user rewards up to date for one user.
    fn update_user_rewards(&self, user_id: usize) {
        let user_rewards = self.load_updated_user_rewards(user_id);
        self.store_user_reward_data(user_id, &user_rewards);
    }

    /// Computes rewards for all delegators and the node.
    /// Updates storage.
    /// Could cost a lot of gas.
    #[endpoint(computeAllRewards)]
    fn compute_all_rewards(&self) {
        let num_nodes = self.user_data().get_num_users();
        let mut sum_unclaimed = BigUint::zero();

        // user 1 is the node and from 2 on are the other delegators,
        // but load_updated_user_rewards works in all cases
        for user_id in 1..(num_nodes+1) {
            let user_data = self.load_updated_user_rewards(user_id);
            self.store_user_reward_data(user_id, &user_data);
            sum_unclaimed += user_data.unclaimed_rewards;
        }

        // divisions are inexact so a small remainder can remain after distributing rewards
        // give it to the validator user, to keep things clear
        let remainder = self.get_total_cumulated_rewards() - sum_unclaimed - self.get_sent_rewards();
        if remainder > 0 {
            let mut node_unclaimed = self.get_user_rew_unclaimed(OWNER_USER_ID);
            node_unclaimed += &remainder;
            self.set_user_rew_unclaimed(OWNER_USER_ID, &node_unclaimed);
        }
    }

    /// Yields how much a user is able to claim in rewards at the present time.
    /// Does not update storage.
    #[view(getClaimableRewards)]
    fn get_claimable_rewards(&self, user: Address) -> BigUint {
        let user_id = self.user_data().get_user_id(&user);
        if user_id == 0 {
            return BigUint::zero()
        }

        let user_data = self.load_updated_user_rewards(user_id);
        user_data.unclaimed_rewards
    }

    /// Utility readonly function to check how many unclaimed rewards currently reside in the contract.
    #[view(getTotalUnclaimedRewards)]
    fn get_total_unclaimed_rewards(&self) -> BigUint {
        let num_nodes = self.user_data().get_num_users();
        let mut sum_unclaimed = BigUint::zero();
        
        // regular rewards
        for user_id in 1..(num_nodes+1) {
            let user_data = self.load_updated_user_rewards(user_id);
            sum_unclaimed += user_data.unclaimed_rewards;
        }

        sum_unclaimed
    }

    /// Retrieve those rewards to which the caller is entitled.
    /// Will send:
    /// - new rewards
    /// - rewards that were previously computed but not sent
    #[endpoint(claimRewards)]
    fn claim_rewards(&self) -> SCResult<()> {
        let caller = self.get_caller();
        let user_id = self.user_data().get_user_id(&caller);
        if user_id == 0 {
            return sc_error!("unknown caller")
        }

        let mut user_data = self.load_updated_user_rewards(user_id);
        
        if user_data.unclaimed_rewards > 0 {
            self.send_rewards(&caller, &user_data.unclaimed_rewards);
            user_data.unclaimed_rewards = BigUint::zero();
        }

        self.store_user_reward_data(user_id, &user_data);

        Ok(())
    }

    fn send_rewards(&self, to: &Address, amount: &BigUint) {
        // send funds
        self.send_tx(to, amount, "delegation rewards claim");

        // increment globally sent funds
        let mut sent_rewards = self.get_sent_rewards();
        sent_rewards += amount;
        self.set_sent_rewards(&sent_rewards);
    }

    /// Loads the entire UserRewardData object from storage.
    fn load_user_reward_data(&self, user_id: usize) -> UserRewardData<BigUint> {
        let u_rew_checkp = self.get_user_rew_checkpoint(user_id);
        let u_rew_unclmd = self.get_user_rew_unclaimed(user_id);
        UserRewardData {
            reward_checkpoint: u_rew_checkp,
            unclaimed_rewards: u_rew_unclmd,
        }
    }

    /// Saves a UserRewardData object to storage.
    fn store_user_reward_data(&self, user_id: usize, data: &UserRewardData<BigUint>) {
        self.set_user_rew_checkpoint(user_id, &data.reward_checkpoint);
        self.set_user_rew_unclaimed(user_id, &data.unclaimed_rewards);
    }

}