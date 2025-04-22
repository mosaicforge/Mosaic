use futures::TryStreamExt;
use grc20_core::{
    mapping::QueryStream, pb::geo,
};
use grc20_sdk::models::space;

mod common;

#[test_log::test(tokio::test)]
async fn test_handle_subspace_added() {
    // Setup Neo4j and IPFS mock
    let (_container, neo4j) = common::neo4j::setup_neo4j().await;
    let (_server, ipfs_client) = common::ipfs_mock::setup_ipfs_mock();
    
    // Create handler
    let handler = common::create_handler(neo4j.clone(), ipfs_client).unwrap();
    let block = common::create_block_metadata();
    
    // Create parent space
    let parent_dao_address = "0x1234567890123456789012345678901234567890";
    let parent_plugin_address = "0xabcdefabcdefabcdefabcdefabcdefabcdefabcd";
    
    let parent_space_created = geo::GeoSpaceCreated {
        dao_address: parent_dao_address.to_string(),
        space_address: parent_plugin_address.to_string(),
    };
    
    let parent_space_id = handler.handle_space_created(&parent_space_created, &[], &block).await.unwrap();
    
    // Create subspace
    let subspace_dao_address = "0x9876543210987654321098765432109876543210";
    let subspace_plugin_address = "0xfedcbafedcbafedcbafedcbafedcbafedcbafedcba";
    
    let subspace_created = geo::GeoSpaceCreated {
        dao_address: subspace_dao_address.to_string(),
        space_address: subspace_plugin_address.to_string(),
    };
    
    let subspace_id = handler.handle_space_created(&subspace_created, &[], &block).await.unwrap();
    
    // Create subspace added event
    let subspace_added = geo::SubspaceAdded {
        subspace: subspace_dao_address.to_string(),
        plugin_address: parent_plugin_address.to_string(),
        change_type: "add".to_string(),
        dao_address: parent_dao_address.to_string(),
    };
    
    // Call handler
    handler.handle_subspace_added(&subspace_added, &block).await.unwrap();
    
    // Verify parent-child relationship was created
    let parent_spaces = space::parent_spaces(&neo4j, &subspace_id)
        .send()
        .await
        .unwrap()
        .try_collect::<Vec<_>>()
        .await
        .expect("Failed to collect parent spaces");
    
    assert_eq!(parent_spaces.len(), 1);
    assert_eq!(parent_spaces[0].0, parent_space_id);
    
    // Also verify using subspaces query
    let subspaces = space::subspaces(&neo4j, &parent_space_id)
        .send()
        .await
        .unwrap()
        .try_collect::<Vec<_>>()
        .await
        .expect("Failed to collect subspaces");
    
    assert_eq!(subspaces.len(), 1);
    assert_eq!(subspaces[0].0, subspace_id);
}

#[test_log::test(tokio::test)]
async fn test_handle_subspace_removed() {
    // Setup Neo4j and IPFS mock
    let (_container, neo4j) = common::neo4j::setup_neo4j().await;
    let (_server, ipfs_client) = common::ipfs_mock::setup_ipfs_mock();
    
    // Create handler
    let handler = common::create_handler(neo4j.clone(), ipfs_client).unwrap();
    let block = common::create_block_metadata();
    
    // Create parent space
    let parent_dao_address = "0x1234567890123456789012345678901234567890";
    let parent_plugin_address = "0xabcdefabcdefabcdefabcdefabcdefabcdefabcd";
    
    let parent_space_created = geo::GeoSpaceCreated {
        dao_address: parent_dao_address.to_string(),
        space_address: parent_plugin_address.to_string(),
    };
    
    let parent_space_id = handler.handle_space_created(&parent_space_created, &[], &block).await.unwrap();
    
    // Create subspace
    let subspace_dao_address = "0x9876543210987654321098765432109876543210";
    let subspace_plugin_address = "0xfedcbafedcbafedcbafedcbafedcbafedcbafedcba";
    
    let subspace_created = geo::GeoSpaceCreated {
        dao_address: subspace_dao_address.to_string(),
        space_address: subspace_plugin_address.to_string(),
    };
    
    let subspace_id = handler.handle_space_created(&subspace_created, &[], &block).await.unwrap();
    
    // Create parent-child relationship
    let subspace_added = geo::SubspaceAdded {
        subspace: subspace_dao_address.to_string(),
        plugin_address: parent_plugin_address.to_string(),
        change_type: "add".to_string(),
        dao_address: parent_dao_address.to_string(),
    };
    
    handler.handle_subspace_added(&subspace_added, &block).await.unwrap();
    
    // Verify relationship was created
    let parent_spaces_before = space::parent_spaces(&neo4j, &subspace_id)
        .send()
        .await
        .unwrap()
        .try_collect::<Vec<_>>()
        .await
        .expect("Failed to collect parent spaces");
    
    assert_eq!(parent_spaces_before.len(), 1);
    
    // Create subspace removed event
    let subspace_removed = geo::SubspaceRemoved {
        subspace: subspace_dao_address.to_string(),
        plugin_address: parent_plugin_address.to_string(),
        change_type: "remove".to_string(),
        dao_address: parent_dao_address.to_string(),
    };
    
    // Call handler
    handler.handle_subspace_removed(&subspace_removed, &block).await.unwrap();
    
    // Verify relationship was removed
    let parent_spaces_after = space::parent_spaces(&neo4j, &subspace_id)
        .send()
        .await
        .unwrap()
        .try_collect::<Vec<_>>()
        .await
        .expect("Failed to collect parent spaces");
    
    assert_eq!(parent_spaces_after.len(), 0);
    
    // Also verify using subspaces query
    let subspaces_after = space::subspaces(&neo4j, &parent_space_id)
        .send()
        .await
        .unwrap()
        .try_collect::<Vec<_>>()
        .await
        .expect("Failed to collect subspaces");
    
    assert_eq!(subspaces_after.len(), 0);
}
