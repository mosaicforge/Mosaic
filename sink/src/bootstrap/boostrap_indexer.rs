use sdk::{indexer_ids, mapping::Triple, system_ids};

pub fn triples() -> Vec<Triple> {
    vec![
        // System attributes
        Triple::new(
            indexer_ids::CREATED_AT_TIMESTAMP,
            system_ids::NAME_ATTRIBUTE,
            "Created At",
        ),
        Triple::new(
            indexer_ids::CREATED_AT_BLOCK,
            system_ids::NAME_ATTRIBUTE,
            "Created At Block",
        ),
        Triple::new(
            indexer_ids::UPDATED_AT_TIMESTAMP,
            system_ids::NAME_ATTRIBUTE,
            "Updated At",
        ),
        Triple::new(
            indexer_ids::UPDATED_AT_BLOCK,
            system_ids::NAME_ATTRIBUTE,
            "Updated At Block",
        ),
        // Space attributes
        Triple::new(
            indexer_ids::SPACE_GOVERNANCE_TYPE,
            system_ids::NAME_ATTRIBUTE,
            "Space Governance Type",
        ),
        Triple::new(
            indexer_ids::SPACE_DAO_ADDRESS,
            system_ids::NAME_ATTRIBUTE,
            "Space DAO Address",
        ),
        Triple::new(
            indexer_ids::SPACE_PLUGIN_ADDRESS,
            system_ids::NAME_ATTRIBUTE,
            "Space Plugin Address",
        ),
        Triple::new(
            indexer_ids::SPACE_VOTING_PLUGIN_ADDRESS,
            system_ids::NAME_ATTRIBUTE,
            "Space Voting Plugin Address",
        ),
        Triple::new(
            indexer_ids::SPACE_MEMBER_PLUGIN_ADDRESS,
            system_ids::NAME_ATTRIBUTE,
            "Space Member Plugin Address",
        ),
        Triple::new(
            indexer_ids::SPACE_GOVERNANCE_TYPE,
            system_ids::NAME_ATTRIBUTE,
            "Space Kind",
        ),
        // Triple::new(indexer_ids::SPACE_VERSION_COUNTER, system_ids::NAME_ATTRIBUTE, "Space Version Counter"),

        // Member and Editor relations
        Triple::new(
            indexer_ids::MEMBER_RELATION,
            system_ids::NAME_ATTRIBUTE,
            "Member Relation",
        ),
        Triple::new(
            indexer_ids::EDITOR_RELATION,
            system_ids::NAME_ATTRIBUTE,
            "Editor Relation",
        ),
        // Parent space
        Triple::new(
            indexer_ids::PARENT_SPACE,
            system_ids::NAME_ATTRIBUTE,
            "Parent Space",
        ),
        // Voting
        Triple::new(
            indexer_ids::VOTE_CAST_TYPE,
            system_ids::NAME_ATTRIBUTE,
            "Vote Cast",
        ),
        Triple::new(
            indexer_ids::VOTE_TYPE_ATTRIBUTE,
            system_ids::NAME_ATTRIBUTE,
            "Vote Type",
        ),
        // Proposal
        Triple::new(
            indexer_ids::PROPOSAL_TYPE,
            system_ids::NAME_ATTRIBUTE,
            "Proposal Type",
        ),
        Triple::new(
            indexer_ids::ADD_MEMBER_PROPOSAL,
            system_ids::NAME_ATTRIBUTE,
            "Add Member Proposal",
        ),
        Triple::new(
            indexer_ids::REMOVE_MEMBER_PROPOSAL,
            system_ids::NAME_ATTRIBUTE,
            "Remove Member Proposal",
        ),
        Triple::new(
            indexer_ids::ADD_EDITOR_PROPOSAL,
            system_ids::NAME_ATTRIBUTE,
            "Add Editor Proposal",
        ),
        Triple::new(
            indexer_ids::REMOVE_EDITOR_PROPOSAL,
            system_ids::NAME_ATTRIBUTE,
            "Remove Editor Proposal",
        ),
        Triple::new(
            indexer_ids::ADD_SUBSPACE_PROPOSAL,
            system_ids::NAME_ATTRIBUTE,
            "Add Subspace Proposal",
        ),
        Triple::new(
            indexer_ids::REMOVE_SUBSPACE_PROPOSAL,
            system_ids::NAME_ATTRIBUTE,
            "Remove Subspace Proposal",
        ),
        Triple::new(
            indexer_ids::EDIT_PROPOSAL,
            system_ids::NAME_ATTRIBUTE,
            "Edit Proposal",
        ),
        // Proposed account and subspace
        Triple::new(
            indexer_ids::PROPOSED_ACCOUNT,
            system_ids::NAME_ATTRIBUTE,
            "Proposed Account",
        ),
        Triple::new(
            indexer_ids::PROPOSED_SUBSPACE,
            system_ids::NAME_ATTRIBUTE,
            "Proposed Subspace",
        ),
        Triple::new(
            indexer_ids::PROPOSAL_CREATOR,
            system_ids::NAME_ATTRIBUTE,
            "Proposal Creator",
        ),
        // Space > PROPOSALS > Proposal
        Triple::new(
            indexer_ids::PROPOSALS,
            system_ids::NAME_ATTRIBUTE,
            "Proposals",
        ),
    ]
}
