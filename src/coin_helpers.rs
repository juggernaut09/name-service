use cosmwasm_std::{Coin, StdResult, StdError};

pub fn assert_sent_sufficient_coin(sent: &[Coin], required: Option<Coin>) -> StdResult<()> {
    if let Some(required_coin) = required {
        let required_amount = required_coin.amount.u128();
        if required_amount > 0 {
            let sent_sufficient_funds = sent.iter().any(|coin| {
               coin.denom == required_coin.denom && coin.amount.u128() >= required_amount
            });

            return if sent_sufficient_funds {
                Ok(())
            } else {
                Err(StdError::generic_err("Insufficient funds sent"))
            }
        }
    }
    Ok(())
}


