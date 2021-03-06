#![no_std]

mod storage;

use node_storage::types::bls_key::BLSKey;

elrond_wasm::imports!();

#[elrond_wasm_derive::contract]
pub trait AuctionMock: storage::AuctionMockStorage {
    #[init]
    fn init(&self) {}

    #[payable("EGLD")]
    #[endpoint]
    fn stake(
        &self,
        num_nodes: usize,
        #[var_args] bls_keys_signatures_args: VarArgs<MultiArg2<BoxedBytes, BoxedBytes>>,
        #[payment] payment: Self::BigUint,
    ) -> SCResult<MultiResultVec<BoxedBytes>> {
        let bls_keys_signatures = bls_keys_signatures_args.into_vec();
        require!(
            num_nodes == bls_keys_signatures.len(),
            "incorrect number of arguments"
        );

        require!(
            !self.is_staking_failure(),
            "auction smart contract deliberate error"
        );

        let mut new_num_nodes = self.get_num_nodes();
        let expected_payment = Self::BigUint::from(num_nodes) * self.get_stake_per_node();
        require!(
            payment == expected_payment,
            "incorrect payment to auction mock"
        );

        let mut result_err_data: Vec<BoxedBytes> = Vec::new();
        for key_sig_pair in bls_keys_signatures.into_iter() {
            new_num_nodes += 1;
            let (bls_key, bls_sig) = key_sig_pair.into_tuple();
            self.set_stake_bls_key(new_num_nodes, bls_key.as_slice());
            self.set_stake_bls_signature(new_num_nodes, bls_sig.as_slice());

            let err_code = self.get_bls_deliberate_error(bls_key.as_slice());
            if err_code > 0 {
                result_err_data.push(bls_key);
                result_err_data.push(BoxedBytes::from(&[err_code][..]));
            }
        }

        self.set_num_nodes(new_num_nodes);

        Ok(result_err_data.into())
    }

    #[endpoint(unStake)]
    fn unstake_endpoint(
        &self,
        #[var_args] bls_keys: VarArgs<BoxedBytes>,
    ) -> SCResult<MultiResultVec<BoxedBytes>> {
        require!(
            !self.is_staking_failure(),
            "auction smart contract deliberate error"
        );

        let mut result_err_data: Vec<BoxedBytes> = Vec::new();
        for (n, bls_key) in bls_keys.iter().enumerate() {
            self.set_unstake_bls_key(n, bls_key.as_slice());

            let err_code = self.get_bls_deliberate_error(bls_key.as_slice());
            if err_code > 0 {
                result_err_data.push(bls_key.clone());
                result_err_data.push(BoxedBytes::from(&[err_code][..]));
            }
        }

        Ok(result_err_data.into())
    }

    #[endpoint(unStakeNodes)]
    fn unstake_nodes_endpoint(
        &self,
        #[var_args] bls_keys: VarArgs<BoxedBytes>,
    ) -> SCResult<MultiResultVec<BoxedBytes>> {
        self.unstake_endpoint(bls_keys)
    }

    #[endpoint(unBond)]
    fn unbond_endpoint(
        &self,
        #[var_args] bls_keys: VarArgs<BoxedBytes>,
    ) -> SCResult<MultiResultVec<BoxedBytes>> {
        require!(
            !self.is_staking_failure(),
            "auction smart contract deliberate error"
        );

        let mut result_err_data: Vec<BoxedBytes> = Vec::new();
        for (n, bls_key) in bls_keys.iter().enumerate() {
            self.set_unbond_bls_key(n, bls_key.as_slice());

            let err_code = self.get_bls_deliberate_error(bls_key.as_slice());
            if err_code > 0 {
                result_err_data.push(bls_key.clone());
                result_err_data.push(BoxedBytes::from(&[err_code][..]));
            }
        }

        let unbond_stake = Self::BigUint::from(bls_keys.len()) * self.get_stake_per_node();
        self.send().direct_egld(
            &self.blockchain().get_caller(),
            &unbond_stake,
            b"unbond stake",
        );

        Ok(result_err_data.into())
    }

    #[endpoint(unBondNodes)]
    fn unbond_nodes_endpoint(
        &self,
        #[var_args] bls_keys: VarArgs<BoxedBytes>,
    ) -> SCResult<MultiResultVec<BoxedBytes>> {
        self.unbond_endpoint(bls_keys)
    }

    #[endpoint]
    fn claim(&self) -> SCResult<()> {
        Ok(())
    }

    #[payable("EGLD")]
    #[endpoint(unJail)]
    fn unjail_endpoint(
        &self,
        #[var_args] bls_keys: VarArgs<BLSKey>,
        #[payment] _fine_payment: Self::BigUint,
    ) -> SCResult<()> {
        self.set_unjailed(&bls_keys.into_vec());
        Ok(())
    }
}
