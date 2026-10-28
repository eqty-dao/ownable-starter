use cosmwasm_std::{
    Addr, from_json,
    testing::{mock_dependencies, mock_env, mock_info},
};
use ownable::{
    contract::{instantiate, query},
    msg::{InstantiateMsg, QueryMsg},
    state::{METADATA, NETWORK_ID, OWNABLE_INFO, PACKAGE_CID},
};
use ownable_std::{InfoResponse, Metadata};
use serde_json::Value;

fn instantiate_msg() -> InstantiateMsg {
    InstantiateMsg {
        ownable_id: "ownable-1".to_string(),
        package: "bafy-package".to_string(),
        ownable_type: Some("base".to_string()),
        network_id: 1,
    }
}

#[test]
fn instantiate_and_queries_preserve_ownable_state_without_nft_identity() {
    let mut deps = mock_dependencies();
    let sender = mock_info("issuer", &[]);

    instantiate(deps.as_mut(), mock_env(), sender, instantiate_msg()).expect("instantiate ownable");

    assert_eq!(NETWORK_ID.load(deps.as_ref().storage).unwrap(), 1);
    assert_eq!(
        PACKAGE_CID.load(deps.as_ref().storage).unwrap(),
        "bafy-package"
    );
    assert_eq!(
        OWNABLE_INFO.load(deps.as_ref().storage).unwrap().owner,
        Addr::unchecked("issuer")
    );
    assert_eq!(
        OWNABLE_INFO.load(deps.as_ref().storage).unwrap().issuer,
        Addr::unchecked("issuer")
    );

    let info_response =
        query(deps.as_ref(), mock_env(), QueryMsg::GetInfo {}).expect("query ownable info");
    let info: InfoResponse = from_json(info_response.clone()).expect("deserialize ownable info");
    assert_eq!(info.owner, Addr::unchecked("issuer"));
    assert_eq!(info.issuer, Addr::unchecked("issuer"));
    assert_eq!(info.ownable_type.as_deref(), Some("default"));

    let info_json: Value = serde_json::from_slice(info_response.as_slice()).expect("info JSON");
    assert!(info_json.get("nft").is_none());

    let metadata_response =
        query(deps.as_ref(), mock_env(), QueryMsg::GetMetadata {}).expect("query ownable metadata");
    let metadata: Metadata = from_json(metadata_response).expect("deserialize ownable metadata");
    assert_eq!(metadata.name.as_deref(), Some("Ownable"));
    assert_eq!(metadata.description.as_deref(), Some(""));
    assert_eq!(metadata.image, None);
    assert_eq!(METADATA.load(deps.as_ref().storage).unwrap(), metadata);
}
