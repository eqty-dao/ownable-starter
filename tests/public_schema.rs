use cosmwasm_schema::schema_for;
use ownable::msg::InstantiateMsg;
use ownable_std::{InfoResponse, Metadata};
use serde_json::Value;

fn schema_json<T: schemars::JsonSchema>() -> Value {
    serde_json::to_value(schema_for!(T)).expect("serialize schema")
}

#[test]
fn public_schemas_remove_nft_identity_and_keep_display_metadata() {
    let instantiate = schema_json::<InstantiateMsg>();
    let info = schema_json::<InfoResponse>();
    let metadata = schema_json::<Metadata>();

    for schema in [&instantiate, &info] {
        assert!(schema["properties"].get("nft").is_none());
        assert!(schema["definitions"].get("NFT").is_none());
    }

    for property in ["name", "description", "image"] {
        assert!(metadata["properties"][property].is_object());
    }
}
