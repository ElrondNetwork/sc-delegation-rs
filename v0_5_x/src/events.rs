imports!();

/// Contains all events that can be emitted by the contract.
#[elrond_wasm_derive::module(EventsModuleImpl)]
pub trait EventsModule {
    #[event("0x0000000000000000000000000000000000000000000000000000000000000001")]
    fn stake_event(&self, delegator: &Address, amount: &BigUint);

    #[event("0x0000000000000000000000000000000000000000000000000000000000000002")]
    fn unstake_event(&self, delegator: &Address, amount: &BigUint);

    #[event("0x0000000000000000000000000000000000000000000000000000000000000003")]
    fn stake_node_ok_event(&self, _data: ());

    #[event("0x0000000000000000000000000000000000000000000000000000000000000004")]
    fn stake_node_fail_event(&self, _reason: &[u8]);

    #[event("0x0000000000000000000000000000000000000000000000000000000000000005")]
    fn unstake_node_ok_event(&self, _data: ());

    #[event("0x0000000000000000000000000000000000000000000000000000000000000006")]
    fn unstake_node_fail_event(&self, _reason: &[u8]);

    #[event("0x0000000000000000000000000000000000000000000000000000000000000007")]
    fn unbond_node_ok_event(&self, _data: ());

    #[event("0x0000000000000000000000000000000000000000000000000000000000000008")]
    fn unbond_node_fail_event(&self, _reason: &[u8]);

    #[event("0x0000000000000000000000000000000000000000000000000000000000000009")]
    fn claim_rewards_event(&self, user: &Address, amount: &BigUint);
}
