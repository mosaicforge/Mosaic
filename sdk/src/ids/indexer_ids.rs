// System attributes
pub const CREATED_AT_TIMESTAMP: &str = "82nP7aFmHJLbaPFszj2nbx";
pub const CREATED_AT_BLOCK: &str = "59HTYnd2e4gBx2aA98JfNx";
pub const UPDATED_AT_TIMESTAMP: &str = "5Ms1pYq8v8G1RXC3wWb9ix";
pub const UPDATED_AT_BLOCK: &str = "7pXCVQDV9C7ozrXkpVg8RJ";

// Space attributes
pub const SPACE_PLUGIN_ADDRESS: &str = "8WQFgjEoqNYHPS9niQGcfC";
pub const SPACE_VOTING_PLUGIN_ADDRESS: &str = "GMuFbsoSsVmiMcCxc34zZA";
pub const SPACE_MEMBER_PLUGIN_ADDRESS: &str = "AGaTTZWAbEaSrmZYTinQuc";
/// Type of space, e.g., Public or Personal
pub const SPACE_KIND: &str = "DEnbDZp79wgfvUXMGJPXDW";

/// GEO_ACCOUNT > MEMBER_RELATION > INDEXED_SPACE
pub const MEMBER_RELATION: &str = "2oGooh2PEUo8pbdMPqcBrQ";

/// GEO_ACCOUNT > EDITOR_RELATION > INDEXED_SPACE
pub const EDITOR_RELATION: &str = "24TNsjdHUhuC5qtsN13stv";

/// SPACE > PARENT_SPACE > SPACE
pub const PARENT_SPACE: &str = "4jLdUCbpRzdjnpjhViDm2d";

/// Special space for the indexer data
pub const INDEXER_SPACE_ID: &str = "Y7z7eeWdupgZayxG9Lvi2Z";

// Voting
/// GEO_ACCOUNT > VOTE_CAST > PROPOSAL
pub const VOTE_CAST: &str = "PfgzdxPYwDUTBCzkXCT9ga";

// Proposal
pub const PROPOSAL_TYPE: &str = "9No6qfEutiKg1WLeXDv73x";
pub const ADD_MEMBER_PROPOSAL: &str = "6dJ23LRTHRdwqoWhtivRrM";
pub const REMOVE_MEMBER_PROPOSAL: &str = "8dJ23LRTHRdwqoWhtivRrM";
pub const ADD_EDITOR_PROPOSAL: &str = "7W7SE2UTj5YTsQvqSmCfLN";
pub const REMOVE_EDITOR_PROPOSAL: &str = "9W7SE2UTj5YTsQvqSmCfLN";
pub const ADD_SUBSPACE_PROPOSAL: &str = "DcEZrRpmAuwxfw7C5G7gjC";
pub const REMOVE_SUBSPACE_PROPOSAL: &str = "FcEZrRpmAuwxfw7C5G7gjC";
pub const EDIT_PROPOSAL: &str = "GcEZrRpmAuwxfw7C5G7gjC";

/// MEMBERSHIP_PROPOSAL_TYPE > PROPOSED_ACCOUNT > GEO_ACCOUNT
/// EDITORSHIP_PROPOSAL_TYPE > PROPOSED_ACCOUNT > GEO_ACCOUNT
pub const PROPOSED_ACCOUNT: &str = "N95e2Jvfe3VX5d6jc9owY6";

/// SUBSPACE_PROPOSAL_TYPE > PROPOSED_SUBSPACE > INDEXED_SPACE
pub const PROPOSED_SUBSPACE: &str = "5ZVrZv7S3Mk2ATV9LAZAha";

/// INDEXED_SPACE > PROPOSALS > PROPOSAL
pub const PROPOSALS: &str = "3gmeTonVCB6B11p3YF8mj5";

/// PROPOSAL > CREATOR > ACCOUNT
pub const PROPOSAL_CREATOR: &str = "FUS7nfaWo8Ugzv9AxsMPNX";
