use grc20_core::{entity::Entity, indexer_ids, mapping::Query, network_ids, pb::geo};
use grc20_sdk::models::{space, SpaceGovernanceType};
use web3_utils::checksum_address;

mod common;

#[tokio::test]
async fn test_handle_space_created() {
    // Setup Neo4j and IPFS mock
    let (_container, neo4j) = common::neo4j::setup_neo4j().await;
    let (_server, ipfs_client) = common::ipfs_mock::setup_ipfs_mock();

    // Create handler
    let handler = common::create_handler(neo4j.clone(), ipfs_client).unwrap();
    let block = common::create_block_metadata();

    // Create test event
    let space_created = geo::GeoSpaceCreated {
        dao_address: "0x1234567890123456789012345678901234567890".to_string(),
        space_address: "0xabcdefabcdefabcdefabcdefabcdefabcdefabcd".to_string(),
    };

    // Call handler
    let space_id = handler
        .handle_space_created(&space_created, &[], &block)
        .await
        .unwrap();

    // Verify space was created correctly
    let space_entity = space::find_one(&neo4j, &space_id, indexer_ids::INDEXER_SPACE_ID)
        .send()
        .await
        .unwrap();

    assert!(space_entity.is_some());
    let space_entity = space_entity.unwrap();

    let expected = Entity::new(
        space_id,
        space::Space {
            network: network_ids::GEO.to_string(),
            dao_contract_address: checksum_address(&space_created.dao_address),
            space_plugin_address: Some(checksum_address(&space_created.space_address)),
            governance_type: SpaceGovernanceType::Public,
            personal_space_admin_plugin: None,
            voting_plugin_address: None,
            member_access_plugin: None,
        },
    );

    assert_eq!(space_entity, expected);
}

#[tokio::test]
async fn test_handle_personal_space_created() {
    // Setup Neo4j and IPFS mock
    let (_container, neo4j) = common::neo4j::setup_neo4j().await;
    let (_server, ipfs_client) = common::ipfs_mock::setup_ipfs_mock();

    // Create handler
    let handler = common::create_handler(neo4j.clone(), ipfs_client).unwrap();
    let block = common::create_block_metadata();

    // First create a space
    let dao_address = "0x1234567890123456789012345678901234567890";
    let space_address = "0xabcdefabcdefabcdefabcdefabcdefabcdefabcd";

    let space_created = geo::GeoSpaceCreated {
        dao_address: dao_address.to_string(),
        space_address: space_address.to_string(),
    };

    let space_id = handler
        .handle_space_created(&space_created, &[], &block)
        .await
        .unwrap();

    // Now create a personal space admin plugin
    let personal_space_created = geo::GeoPersonalSpaceAdminPluginCreated {
        dao_address: dao_address.to_string(),
        personal_admin_address: "0xfedcbafedcbafedcbafedcbafedcbafedcbafedcba".to_string(),
        initial_editor: "0x9876543210987654321098765432109876543210".to_string(),
    };

    handler
        .handle_personal_space_created(&personal_space_created, &block)
        .await
        .unwrap();

    // Verify space was updated correctly
    let space_entity = space::find_one(&neo4j, &space_id, indexer_ids::INDEXER_SPACE_ID)
        .send()
        .await
        .unwrap();

    assert!(space_entity.is_some());
    let space_entity = space_entity.unwrap();

    let expected = Entity::new(
        space_id,
        space::Space {
            network: network_ids::GEO.to_string(),
            dao_contract_address: checksum_address(&space_created.dao_address),
            space_plugin_address: Some(checksum_address(&space_created.space_address)),
            governance_type: SpaceGovernanceType::Personal,
            personal_space_admin_plugin: Some(checksum_address(
                &personal_space_created.personal_admin_address,
            )),
            voting_plugin_address: None,
            member_access_plugin: None,
        },
    );

    assert_eq!(space_entity, expected);
}

#[tokio::test]
async fn test_handle_governance_plugin_created() {
    // Setup Neo4j and IPFS mock
    let (_container, neo4j) = common::neo4j::setup_neo4j().await;
    let (_server, ipfs_client) = common::ipfs_mock::setup_ipfs_mock();

    // Create handler
    let handler = common::create_handler(neo4j.clone(), ipfs_client).unwrap();
    let block = common::create_block_metadata();

    // First create a space
    let dao_address = "0x1234567890123456789012345678901234567890";
    let space_address = "0xabcdefabcdefabcdefabcdefabcdefabcdefabcd";

    let space_created = geo::GeoSpaceCreated {
        dao_address: dao_address.to_string(),
        space_address: space_address.to_string(),
    };

    let space_id = handler
        .handle_space_created(&space_created, &[], &block)
        .await
        .unwrap();

    // Now create a governance plugin
    let governance_plugin_created = geo::GeoGovernancePluginCreated {
        dao_address: dao_address.to_string(),
        main_voting_address: "0xfedcbafedcbafedcbafedcbafedcbafedcbafedcba".to_string(),
        member_access_address: "0x9876543210987654321098765432109876543210".to_string(),
    };

    handler
        .handle_governance_plugin_created(&governance_plugin_created, &block)
        .await
        .unwrap();

    // Verify space was updated correctly
    let space_entity = space::find_one(&neo4j, &space_id, indexer_ids::INDEXER_SPACE_ID)
        .send()
        .await
        .unwrap();

    assert!(space_entity.is_some());
    let space_entity = space_entity.unwrap();

    let expected = Entity::new(
        space_id,
        space::Space {
            network: network_ids::GEO.to_string(),
            dao_contract_address: checksum_address(&space_created.dao_address),
            space_plugin_address: Some(checksum_address(&space_created.space_address)),
            governance_type: SpaceGovernanceType::Public,
            personal_space_admin_plugin: None,
            voting_plugin_address: Some(checksum_address(
                &governance_plugin_created.main_voting_address,
            )),
            member_access_plugin: Some(checksum_address(
                &governance_plugin_created.member_access_address,
            )),
        },
    );

    assert_eq!(space_entity, expected);
}
