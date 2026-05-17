use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
#[cfg(not(feature = "library"))]
use cosmwasm_std::{Addr, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cosmwasm_std::{Binary, to_json_binary};
use cw2::set_contract_version;
use crate::state::{Config, NFT_ITEM, CONFIG, METADATA, PACKAGE_CID, OWNABLE_INFO, NETWORK_ID};
use ownable_std::{package_title_from_name, InfoResponse, Metadata, OwnableEvent, OwnableInfo, PublicEvent};

// version info for migration info
const CONTRACT_NAME: &str = concat!("crates.io:", env!("CARGO_PKG_NAME"));
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");


pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let ownable_info = OwnableInfo {
        owner: info.sender.clone(),
        issuer: info.sender.clone(),
        ownable_type: Some("default".to_string()),
    };

    let package_title = package_title_from_name(env!("CARGO_PKG_NAME"));
    let metadata = Metadata {
        image: None,
        image_data: None,
        external_url: None,
        description: Some(env!("CARGO_PKG_DESCRIPTION").to_string()),
        name: Some(package_title.clone()),
        background_color: None,
        animation_url: None,
        youtube_url: None
    };

    NETWORK_ID.save(deps.storage, &msg.network_id)?;
    CONFIG.save(deps.storage, &Config {})?;
    if let Some(nft) = msg.nft {
        NFT_ITEM.save(deps.storage, &nft)?;
    }
    METADATA.save(deps.storage, &metadata)?;
    OWNABLE_INFO.save(deps.storage, &ownable_info)?;
    PACKAGE_CID.save(deps.storage, &msg.package)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", ownable_info.owner.clone())
        .add_attribute("issuer", ownable_info.issuer.clone()))
}

pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Transfer { to } => try_transfer(info, deps, to),
    }
}

pub fn try_transfer(info: MessageInfo, deps: DepsMut, to: Addr) -> Result<Response, ContractError> {
    let address = info.sender.clone();

    OWNABLE_INFO.update(deps.storage, |mut config| -> Result<_, ContractError> {
        if address != config.owner {
            return Err(ContractError::Unauthorized {
                val: "Unauthorized transfer attempt".to_string(),
            });
        }
        if address == to {
            return Err(ContractError::CustomError {
                val: "Unable to transfer: Recipient address is current owner".to_string(),
            });
        }
        config.owner = to.clone();
        Ok(config)
    })?;
    Ok(Response::new()
        .add_attribute("method", "try_transfer")
        .add_attribute("new_owner", to.to_string())
    )
}

pub fn register(
    info: MessageInfo,
    deps: DepsMut,
    event: PublicEvent,
) -> Result<Response, ContractError> {
    let _ = (info, deps);
    Err(ContractError::MatchEventError { val: event.event_type })
}

pub fn ingest(
    info: MessageInfo,
    deps: DepsMut,
    event: OwnableEvent,
) -> Result<Response, ContractError> {
    let _ = (info, deps);
    Err(ContractError::MatchEventError { val: event.event_type })
}


pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetInfo {} => query_ownable_info(deps),
        QueryMsg::GetMetadata {} => query_ownable_metadata(deps),
        QueryMsg::GetWidgetState {} => query_ownable_widget_state(deps),
    }
}

fn query_ownable_widget_state(deps: Deps) -> StdResult<Binary> {
    let widget_config = CONFIG.load(deps.storage)?;
    to_json_binary(&widget_config)
}

fn query_ownable_info(deps: Deps) -> StdResult<Binary> {
    let nft = NFT_ITEM.may_load(deps.storage)?;
    let ownable_info = OWNABLE_INFO.load(deps.storage)?;
    to_json_binary(&InfoResponse {
        owner: ownable_info.owner,
        issuer: ownable_info.issuer,
        nft,
        ownable_type: ownable_info.ownable_type,
    })
}

fn query_ownable_metadata(deps: Deps) -> StdResult<Binary> {
    let cw721 = METADATA.load(deps.storage)?;
    to_json_binary(&Metadata {
        image: cw721.image,
        image_data: cw721.image_data,
        external_url: cw721.external_url,
        description: cw721.description,
        name: cw721.name,
        background_color: cw721.background_color,
        animation_url: cw721.animation_url,
        youtube_url: cw721.youtube_url,
    })
}

#[cfg(test)]
mod tests {
    use super::{ingest, register};
    use cosmwasm_std::testing::{mock_dependencies, mock_info};
    use ownable_std::{OwnableEvent, OwnableEventSource, PublicEvent};
    use serde_json::json;

    #[test]
    fn register_rejects_all_event_types() {
        let mut deps = mock_dependencies();
        let event = PublicEvent {
            source: "0xsource".to_string(),
            event_type: "consume".to_string(),
            data: vec![0x01, 0x02].into(),
            block_number: 1,
            transaction_hash: vec![0xaa].into(),
            transaction_index: 0,
            log_index: 0,
        };

        let err = register(mock_info("owner", &[]), deps.as_mut(), event).unwrap_err();
        assert_eq!(err.to_string(), "Unknown event type: \"consume\"");
    }

    #[test]
    fn ingest_rejects_all_event_types() {
        let mut deps = mock_dependencies();
        let event = OwnableEvent {
            source: OwnableEventSource {
                id: "source-id".to_string(),
                owner: "owner".to_string(),
                issuer: "issuer".to_string(),
            },
            event_type: "consume".to_string(),
            attributes: json!({"amount": 1}),
        };

        let err = ingest(mock_info("owner", &[]), deps.as_mut(), event).unwrap_err();
        assert_eq!(err.to_string(), "Unknown event type: \"consume\"");
    }
}
