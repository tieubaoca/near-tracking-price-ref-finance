use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{UnorderedMap};
use near_sdk::json_types::{ValidAccountId, U128};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    env, ext_contract, near_bindgen, AccountId, 
    PanicOnDefault, PromiseOrValue, PromiseResult, 
};
near_sdk::setup_alloc!();

pub const REF_FINANCE: &str = "ref-finance.testnet";
pub const WNEAR: &str = "wrap.testnet";

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    token_to_ref_pool_id: UnorderedMap<AccountId, UnorderedMap<AccountId, u64>>,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new() -> Self {
        Self {
            token_to_ref_pool_id: UnorderedMap::new(b"token".to_vec()),
        }
    }

    pub fn add_pool(&mut self, token_1: ValidAccountId, token_2: ValidAccountId, pool_id: u64) {
        self.internal_add_pool(token_1.clone(), token_2.clone(), pool_id);
        self.internal_add_pool(token_2, token_1, pool_id);
    }
    pub fn get_return(
        &self,
        token_in: ValidAccountId,
        amount_in: U128,
        token_out: ValidAccountId,
    ) -> PromiseOrValue<U128> {
        if token_in == ValidAccountId::try_from(WNEAR).unwrap()
            || token_out == ValidAccountId::try_from(WNEAR).unwrap()
        {
            let pool_wnear = self
                .token_to_ref_pool_id
                .get(&token_in.clone().into())
                .unwrap()
                .get(&WNEAR.to_string())
                .unwrap();
            PromiseOrValue::from(
                ref_contract::get_return(
                    pool_wnear,
                    token_in,
                    U128::from(amount_in),
                    ValidAccountId::try_from(WNEAR).unwrap(),
                    &REF_FINANCE,
                    0,
                    5_000_000_000_000,
                ))
        } else {
            let pool_in_wnear = self
                .token_to_ref_pool_id
                .get(&token_in.clone().into())
                .unwrap()
                .get(&WNEAR.to_string())
                .unwrap();
            PromiseOrValue::from(
                ref_contract::get_return(
                    pool_in_wnear,
                    token_in,
                    U128::from(amount_in),
                    ValidAccountId::try_from(WNEAR).unwrap(),
                    &REF_FINANCE,
                    0,
                    5_000_000_000_000,
                )
                .then(ext_self::get_return_token_out(
                    token_out,
                    &env::current_account_id(),
                    0,
                    15_000_000_000_000,
                )),
            )
        }
    }

    fn internal_add_pool(
        &mut self,
        token_1: ValidAccountId,
        token_2: ValidAccountId,
        pool_id: u64,
    ) {
        let mut mapping = self
            .token_to_ref_pool_id
            .get(&token_1.clone().into())
            .unwrap_or(UnorderedMap::new(
                format!("1{}{}", token_1, token_2).as_bytes(),
            ));
        mapping.insert(&token_2.clone().into(), &pool_id);
        self.token_to_ref_pool_id
            .insert(&token_1.clone().into(), &mapping);
    }

    pub fn get_return_token_out(&self, token_out: ValidAccountId) -> PromiseOrValue<U128> {
        if let PromiseResult::Successful(result) =
            env::promise_result(env::promise_results_count() - 1u64)
        {
            let amount_near_out = near_sdk::serde_json::from_slice::<U128>(&result).unwrap();
            let pool_out_wnear = self
                .token_to_ref_pool_id
                .get(&token_out.clone().into())
                .unwrap()
                .get(&WNEAR.to_string())
                .unwrap();
            PromiseOrValue::from(ref_contract::get_return(
                pool_out_wnear,
                ValidAccountId::try_from(WNEAR).unwrap(),
                U128::from(amount_near_out),
                token_out,
                &REF_FINANCE,
                0,
                5_000_000_000_000,
            ))
        } else {
            env::panic(b"fail!");
        }
    }

    // pub fn get_price_promise(&self, token: ValidAccountId) -> U128 {
    //     ref_contract::get_return(
    //         153,
    //         token,
    //         U128::from(10000),
    //         ValidAccountId::try_from("v2.wnear.flux-dev").unwrap(),
    //         &REF_FINANCE,
    //         0,
    //         5_000_000_000_000,
    //     )
    // }
}

#[ext_contract(ext_self)]
trait TSelf {
    fn get_return_token_out(&self, token_out: ValidAccountId) -> PromiseOrValue<U128>;
}

#[ext_contract(ref_contract)]
trait TRefFinance {
    #[payable]
    fn swap(&mut self, actions: Vec<SwapAction>, referral_id: Option<ValidAccountId>) -> U128;
    fn get_pool(&self, pool_id: u64) -> PoolInfo;
    fn get_return(
        &self,
        pool_id: u64,
        token_in: ValidAccountId,
        amount_in: U128,
        token_out: ValidAccountId,
    ) -> U128;
}
/// Single swap action.
#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct SwapAction {
    /// Pool which should be used for swapping.
    pub pool_id: u64,
    /// Token to swap from.
    pub token_in: AccountId,
    /// Amount to exchange.
    /// If amount_in is None, it will take amount_out from previous step.
    /// Will fail if amount_in is None on the first step.
    pub amount_in: Option<U128>,
    /// Token to swap into.
    pub token_out: AccountId,
    /// Required minimum amount of token_out.
    pub min_amount_out: U128,
}
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
