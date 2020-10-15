use cosmwasm_std::{to_binary, Api, Binary, Env, Extern, HandleResponse, InitResponse, MessageInfo, Querier, StdResult, Storage, HumanAddr, HandleResult, StdError};

use crate::msg::{HandleMsg, InitMsg, QueryMsg, ResolveRecordResponse};
use crate::state::{config, config_read, State, NameRecord, resolver, resolver_read};
use crate::coin_helpers::assert_sent_sufficient_coin;

const MIN_NAME_LENGTH: usize = 3;
const MAX_NAME_LENGTH: usize = 64;

// Note, you can use StdResult in some functions where you do not
// make use of the custom errors
pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    _env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    let state = State {
        purchase_price: msg.purchase_price,
        transfer_price: msg.transfer_price
    };
    config(&mut deps.storage).save(&state)?;

    Ok(InitResponse::default())
}

// And declare a custom Error variant for the ones where you will want to make use of it
pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    info: MessageInfo,
    msg: HandleMsg,
) -> HandleResult {
    match msg {
        HandleMsg::Register { name } => try_register(deps, info, name),
        HandleMsg::Transfer { name, to } => try_transfer(deps, env, info, name, to),
    }
}

pub fn try_register<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    info: MessageInfo,
    name: String,
) -> HandleResult {
    // we only need to check here - at point of registration
    validate_name(&name)?;
    let config_state = config(&mut deps.storage).load()?;
    assert_sent_sufficient_coin(&info.sent_funds, config_state.purchase_price)?;

    let key = name.as_bytes();
    let record = NameRecord{
        owner: deps.api.canonical_address(&info.sender)?,
    };

    if (resolver(&mut deps.storage).may_load(key)?).is_some() {
        // name is already taken
        return Err(StdError::generic_err("Name is already taken"));
    }

    // name is available
    resolver(&mut deps.storage).save(key, &record)?;

    Ok(HandleResponse::default())
}

pub fn try_transfer<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    _env: Env,
    info: MessageInfo,
    name: String,
    to: HumanAddr,
) -> HandleResult {
    let api = deps.api;
    let config_state = config(&mut deps.storage).load()?;
    assert_sent_sufficient_coin(&info.sent_funds, config_state.purchase_price)?;
    let key = name.into_bytes();
    let new_owner= deps.api.canonical_address(&to)?;

    resolver(&mut deps.storage).update(&key, |record| {
       if let Some(mut record) = record {
             if api.canonical_address(&info.sender)? != record.owner {
                 return Err(StdError::generic_err("Sender must be the owner of the address"))
             }
           record.owner = new_owner.clone();
           Ok(record)
       } else {
           Err(StdError::generic_err("Name does not exist"))
       }
    })?;
    Ok(HandleResponse::default())
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    _env: Env,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::ResolveRecord {name} => query_resolver(deps, name),
        QueryMsg::Config {} => to_binary(&config_read(&deps.storage).load()?),
    }
}

fn query_resolver<S: Storage, A: Api, Q: Querier>(
deps: &Extern<S, A, Q>,
name: String,
) -> StdResult<Binary> {
let key = name.as_bytes();

let address = match resolver_read(&deps.storage).may_load(key)? {
Some(record) => Some(deps.api.human_address(&record.owner)?),
None => None,
};
let resp = ResolveRecordResponse { address };

to_binary(&resp)
}

// let's not import a regexp library and just do these checks by hand
fn invalid_char(c: char) -> bool {
    let is_valid =
        (c >= '0' && c <= '9') || (c >= 'a' && c <= 'z') || (c == '.' || c == '-' || c == '_');
    !is_valid
}

/// validate_name returns an error if the name is invalid
/// (we require 3-64 lowercase ascii letters, numbers, or . - _)
fn validate_name(name: &str) -> StdResult<()> {
    if name.len() < MIN_NAME_LENGTH {
        Err(StdError::generic_err("Name too short"))
    } else if name.len() > MAX_NAME_LENGTH {
        Err(StdError::generic_err("Name too long"))
    } else {
        match name.find(invalid_char) {
            None => Ok(()),
            Some(bytepos_invalid_char_start) => {
                let c = name[bytepos_invalid_char_start..].chars().next().unwrap();
                Err(StdError::generic_err(format!("Invalid character: '{}'", c)))
            }
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
//     use cosmwasm_std::{coins, from_binary};
//
//     #[test]
//     fn proper_initialization() {
//         let mut deps = mock_dependencies(&[]);
//
//         let msg = InitMsg { count: 17 };
//         let info = mock_info("creator", &coins(1000, "earth"));
//
//         // we can just call .unwrap() to assert this was a success
//         let res = init(&mut deps, mock_env(), info, msg).unwrap();
//         assert_eq!(0, res.messages.len());
//
//         // it worked, let's query the state
//         let res = query(&deps, mock_env(), QueryMsg::GetCount {}).unwrap();
//         let value: CountResponse = from_binary(&res).unwrap();
//         assert_eq!(17, value.count);
//     }
//
//     #[test]
//     fn increment() {
//         let mut deps = mock_dependencies(&coins(2, "token"));
//
//         let msg = InitMsg { count: 17 };
//         let info = mock_info("creator", &coins(2, "token"));
//         let _res = init(&mut deps, mock_env(), info, msg).unwrap();
//
//         // beneficiary can release it
//         let info = mock_info("anyone", &coins(2, "token"));
//         let msg = HandleMsg::Increment {};
//         let _res = handle(&mut deps, mock_env(), info, msg).unwrap();
//
//         // should increase counter by 1
//         let res = query(&deps, mock_env(), QueryMsg::GetCount {}).unwrap();
//         let value: CountResponse = from_binary(&res).unwrap();
//         assert_eq!(18, value.count);
//     }
//
//     #[test]
//     fn reset() {
//         let mut deps = mock_dependencies(&coins(2, "token"));
//
//         let msg = InitMsg { count: 17 };
//         let info = mock_info("creator", &coins(2, "token"));
//         let _res = init(&mut deps, mock_env(), info, msg).unwrap();
//
//         // beneficiary can release it
//         let unauth_info = mock_info("anyone", &coins(2, "token"));
//         let msg = HandleMsg::Reset { count: 5 };
//         let res = handle(&mut deps, mock_env(), unauth_info, msg);
//         match res {
//             Err(ContractError::Unauthorized {}) => {}
//             _ => panic!("Must return unauthorized error"),
//         }
//
//         // only the original creator can reset the counter
//         let auth_info = mock_info("creator", &coins(2, "token"));
//         let msg = HandleMsg::Reset { count: 5 };
//         let _res = handle(&mut deps, mock_env(), auth_info, msg).unwrap();
//
//         // should now be 5
//         let res = query(&deps, mock_env(), QueryMsg::GetCount {}).unwrap();
//         let value: CountResponse = from_binary(&res).unwrap();
//         assert_eq!(5, value.count);
//     }
// }

