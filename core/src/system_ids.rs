// ================================================================
// Ported from https://github.com/geobrowser/geogenesis/blob/stream/1.0.0/packages/sdk/src/system-ids.ts
// ================================================================

pub const IMAGE_ATTRIBUTE: &str = "457a27af7b0b485cac07aa37756adafa";
pub const DESCRIPTION: &str = "9b1f76ff9711404c861e59dc3fa7d037";
pub const NAME: &str = "a126ca530c8e48d5b88882c734c38935";
/// Space attribute: Entity.SPACE = Space.id
pub const SPACE: &str = "362c1dbddc6444bba3c4652f38a642d7";
/// Space type: SomeSpace: INDEXED_SPACE
pub const INDEXED_SPACE: &str = "306598522df542f69ad72921c33ad84b";
pub const ATTRIBUTE: &str = "808a04ceb21c4d888ad12e240613e5ca";

/**
 * Relations are a data model that enable us to create references between some
 * arbitrary id and a set of entity ids.
 *
 * They act similarly to Join Tables in a relational database, but are designed
 * around the graph-based nature of the Geo data model.
 *
 * Relations are themselves entities, so can store any metadata about the relation
 * as triples. Currently Relation entities cannot have their own relations. This is a
 * technical limitation to avoid infinitely creating recursive relations.
 *
 * ┌─────────────────────┐       ┌────────────────────┐      ┌──────────────────────┐
 * │                     │       │                    │      │                      │
 * │      Entity         │◄──────┤     Relation       │─────►│        Entity        │
 * │                     │       │                    │      │                      │
 * └─────────────────────┘       └────────────────────┘      └──────────────────────┘
 */

/**
 * Relation type. This is the entity representing the Join between the
 * the Collection and the Entity
 */
pub const RELATION: &str = "c167ef23fb2a40449ed945123ce7d2a9";

/**
 * Relation's from reference. This is the attribute that references
 * the Collection id
 */
pub const RELATION_FROM_ATTRIBUTE: &str = "c43b537bcff742718822717fdf2c9c01";

/**
 * Relation to reference. This is the attribute that references
 * the Entity id for a given Collection item entry
 */
pub const RELATION_TO_ATTRIBUTE: &str = "c1f4cb6fece44c3ca447ab005b756972";

/**
 * The type of the Relation. e.g., Type, Attribute, Friend, Married to
 */
pub const RELATION_TYPE_ATTRIBUTE: &str = "d747a35a6aa14f468f76e6c2064c7036";
// pub const RELATION_TYPE_OF_ATTRIBUTE: &str = "c167ef23fb2a40449ed945123ce7d277'

/**
 * Collection item's ordering within the collection. Collections are unordered by
 * default, but we set all collection items to a default index value of 0.
 */
pub const RELATION_INDEX: &str = "ede47e6930b044998ea4aafbda449609";

/**
 * Collection entity type. This is used when the Collection itself is an entity
 * vs. being a value in a Triple
 */
pub const COLLECTION_TYPE: &str = "c373a33052df47b3a6d2df552bda4b44";

/**
 * Data entities in Geo can specify one or many data sources. These data sources
 * might fall into various categories which determine how data for these sources
 * are fetched.
 *
 * A collection data source points to a collection with one or many collection item
 * relations coming from it.
 *
 * A query data source points to one, many, or no spaces. This determines which spaces
 * we query data from.
 *
 * An all-of-geo data source doesn't point to any spaces, and instead queries the
 * entirety of the knowledge graph.
 */
pub const DATA_SOURCE_ATTRIBUTE: &str = "f971040607704042947558595625e48f";
pub const DATA_SOURCE_TYPE_RELATION_TYPE: &str = "aa0d4ddd70994b1093339fd7a8e5f715";
pub const COLLECTION_DATA_SOURCE: &str = "a82dd96e24064feea770265ddf707ee3";
pub const QUERY_DATA_SOURCE: &str = "8c3658dd8b174cb9836daf9278c179e2";
pub const ALL_OF_GEO_DATA_SOURCE: &str = "21417aaa69b745509f4297e59ffd8e2b";

/**
 * The collection item relation type is used to identify the relations that point to
 * collection items from a collection.
 */
pub const COLLECTION_ITEM_RELATION_TYPE: &str = "66579048ca0d47b1-8ac1c9de1ddfd4bd";

/*
 * Example Usage: Rhonda Patrick > TYPES > Person
 * Note that we should probably convert "type" to "types" or a UUID in the future.
 */
pub const TYPES: &str = "8f151ba4de204e3c9cb499ddf96f48f1";

/* Example Usage: Person > ATTRIBUTES > Age */
pub const ATTRIBUTES: &str = "01412f8381894ab1836565c7fd358cc1";

// A Type is a categorization of an Entity. For example, a Person Type has specific
// schema associated with it. An Entity can be a Person Type. An Entity can have
// multiple Types. For example, an Entity can be a Person and a Philosopher.
/* Example Usage: Person > TYPES > SCHEMA_TYPE */
// Note: This is the type type
pub const SCHEMA_TYPE: &str = "d7ab40920ab5441e88c35c27952de773";

pub const VALUE_TYPE: &str = "ee26ef23f7f14eb6b7423b0fa38c1fd8";

/* Example Usage: Thumbnail > VALUE_TYPE > IMAGE */
pub const IMAGE: &str = "ba4e41460010499da0a3caaa7f579d0e";
pub const IMAGE_WIDTH_ATTRIBUTE: &str = "18a7f15ea93b4e15bacf4d57052741e9";
pub const IMAGE_HEIGHT_ATTRIBUTE: &str = "58747e352a1c4c76ae64bfe08d28d0a4";
pub const IMAGE_FILE_TYPE_ATTRIBUTE: &str = "03d3a32b258f492e8d81c9ee2bc01461";
pub const IMAGE_URL_ATTRIBUTE: &str = "334b8ac01be14079b1707e11d0f9eb8d";

/* Example Usage: City > VALUE_TYPE > RELATION */
pub const RELATION_TYPE: &str = "14611456b4664cab920d2245f59ce828";
pub const COLLECTION_VALUE_TYPE: &str = "v4611456b4444cab920d2245f59ce828";

/* Example Usage: Address > VALUE_TYPE > TEXT */
pub const TEXT: &str = "9edb6fcce4544aa5861139d7f024c010";
pub const NUMBER: &str = "9b597aaec31c46c88565a370da0c2a65";
pub const CHECKBOX: &str = "7aa4792eeacd41868272fa7fc18298ac";

// Date of Birth > VALUE_TYPE > DATE
pub const DATE: &str = "167664f668f840e1976b20bd16ed8d47";

// Twitter > VALUE_TYPE > WEB_URL
pub const WEB_URL: &str = "dfc221d98cce4f0b9353e437a98387e3";

// This sets the type of values which can be set as part of a relation value.
// e.g. An attribute called People can only accept values of type Person
pub const RELATION_VALUE_RELATIONSHIP_TYPE: &str = "cfa6a2f5151f43bfa684f7f0228f63ff";

/* Note that this is a temporary workaround for production MVP release. As such, this system ID isn't included in the bootstrap process.*/
pub const DEFAULT_TYPE: &str = "aeebbd5e4d794d24ae99239e9142d9ed";

pub const PERSON_TYPE: &str = "af7ae93b97d64aedad690c1d3da149a1";
pub const COMPANY_TYPE: &str = "9cc8a65ddf924c0c8d9024980e822dc0";
pub const NONPROFIT_TYPE: &str = "b3b03c909b6d487cb2e2a7d685f120eb";
pub const PROJECT_TYPE: &str = "cb9d261d456b4eaf87e51e9faa441867";
pub const REGION_TYPE: &str = "911a8e0a52f24655a0c6d89cd161bb12";
pub const NONPROFIT_SERVICE_TYPE: &str = "2edf4225793741bab2056ac91ab4aab4";
pub const FINANCE_OVERVIEW_TYPE: &str = "2cc9d24459ea427f9257f1362a5fa952";
pub const FINANCE_SUMMMARY_TYPE: &str = "ce59ccc12ac54ace8f8209322434733d";
pub const TAG_TYPE: &str = "3d31f766b65148afa357271343a773de";
pub const TOPIC_TYPE: &str = "1d7f027e415c4f69800e460fde65feb9";
pub const GOAL_TYPE: &str = "f71912463dca4e778a79d9cdc9804127";
pub const CLAIM_TYPE: &str = "fa8e8e54f7424c00b73c05adee2b4545";

pub const PAGE_TYPE: &str = "1a9fc4a00fec4eeaa075eec7ebd0d043";
pub const TAB_TYPE: &str = "2c72ace7540444559d2265272a94e874";
pub const POST_TYPE: &str = "682fbeff41e242cda7f9c4909136a8c5";
pub const PAGE_TYPE_TYPE: &str = "5ec8adc335334c3cbfa4acdfaa877bac";

pub const VALUES_ATTRIBUTE: &str = "c8e8fd5f011d4c8e8aaf1a2ffc5b48fd";
pub const VISION_ATTRIBUTE: &str = "c670247893c74af48f2a285a46cc19ca";
pub const MISSION_ATTRIBUTE: &str = "6db5eaa51cf6463e88f987bd631db044";
pub const SPEAKERS_ATTRIBUTE: &str = "03597522e1f2423b882d330cfe89331d";
pub const RELEVANT_QUESTIONS_ATTRIBUTE: &str = "ee5648a5d63847809796cd8605517545";
pub const SUPPORTING_ARGUMENTS_ATTRIBUTE: &str = "cd598fe88dc540fbafc727d363aa2b31";
pub const BROADER_CLAIMS_ATTRIBUTE: &str = "8db09ed21a66408eab8d8e8f931a09cf";
pub const SOURCES_ATTRIBUTE: &str = "5b4e9b7455f44e57b0b358da71188191";
pub const QUOTES_ATTRIBUTE: &str = "4ca754c9a01a4ef2a5d6597c58764529";
pub const SUBCLAIMS_ATTRIBUTE: &str = "21d9fa3cfecf42bc8f8e9fcc6ae2b0cd";
pub const OPPOSING_ARGUMENTS_ATTRIBUTE: &str = "0c0a2a9519284ec4876dcc04075b7927";
pub const BROADER_TOPICS_ATTRIBUTE: &str = "9c2ef1313a1547e9ac5d0fce07e792a1";
pub const DEFINITIONS_ATTRIBUTE: &str = "37ae1d79b26e4bf588cb69087a992dc9";
pub const RELATED_TOPICS_ATTRIBUTE: &str = "0db47aca1ccf4c9fbeb689519ebe9eed";
pub const SUBTOPICS_ATTRIBUTE: &str = "21be6a84312544a2bb2e3c23928ce4aa";
pub const TAGS_ATTRIBUTE: &str = "90dcfc330cdb4252a7c3f653d4f54e26";
pub const CLAIMS_FROM_ATTRIBUTE: &str = "7fa816a3cb704534934888449869dc33";
pub const SUBGOALS_ATTRIBUTE: &str = "377ac7e818ab443cbc2629ff04745f99";
pub const BROADER_GOALS_ATTRIBUTE: &str = "2bd0960f5af94b0c893920e9edf31ede";
pub const PERSON_ATTRIBUTE: &str = "626e4ad561c349aeaf5e3c80e53cf890";
pub const TOPICS_ATTRIBUTE: &str = "5742a7038b734eb6b3df4378c1b512c6";
pub const REGION_ATTRIBUTE: &str = "5e4911b82093411ea445bc2124d7f8e3";
pub const EMAIL_ATTRIBUTE: &str = "a89fcd1081b343e48f770d9561a68acd";
pub const STREET_ADDRESS_ATTRIBUTE: &str = "c4b9a30a92a945748f9b31c41eb8bbd8";
pub const PHONE_NUMBER_ATTRIBUTE: &str = "cb36140946954676b62fc2290613a430";
pub const NONPROFIT_ID_NUMBER_ATTRIBUTE: &str = "dcb87494cb91447b9a04625bd1acc804";
pub const GOALS_ATTRIBUTE: &str = "f9804f7c0e2e4658a8489aa65bbe411b";
pub const NONPROFIT_CATEGORIES_ATTRIBUTE: &str = "fca2a465642640bb8e7ecf33742b5346";
pub const WEB_URL_ATTRIBUTE: &str = "e8010874d3304a4d990762e89a19371a";
pub const AVATAR_ATTRIBUTE: &str = "235ba0e8dc7e4bdda1e16d0d4497f133";
pub const COVER_ATTRIBUTE: &str = "34f535072e6b42c5a84443981a77cfa2";
pub const ROLE_ATTRIBUTE: &str = "9c1922f1d7a247d1841d234cb2f56991";

/* Example Usage: SF_Config > FOREIGN_TYPES > Some_Entity */
pub const FOREIGN_TYPES: &str = "be74597305a94cd0a46d1c5538270faf";

/* Example Usage: SF Config > TYPES > SPACE_CONFIGURATION */
pub const SPACE_CONFIGURATION: &str = "1d5d0c2adb23466ca0b09abe879df457";
pub const SOURCE_SPACE_ATTRIBUTE: &str = "315c23ed14414c378f14bd8b64f36702";

/* Example Usage: Block Entity > TYPES > TABLE_BLOCK */
pub const TABLE_BLOCK: &str = "88d5925217ae4d9aa36724710129eb47";

pub const SHOWN_COLUMNS: &str = "388ad59b1cc7413ca0bb34a4de48c758";
pub const PLACEHOLDER_TEXT: &str = "0e5f84e4c85a44698a665a7d46fe2786";
pub const PLACEHOLDER_IMAGE: &str = "3f20832090704795a046206a6efb9557";

pub const VIEW_TYPE: &str = "2a734759874246efaac4c16b53f3a542";
pub const VIEW_ATTRIBUTE: &str = "f062fc5a6f114859ba70e644be6caea5";
pub const TABLE_VIEW: &str = "a2a136e1d1da4853bf3b0960982f8162";
pub const LIST_VIEW: &str = "70db74421c6e425291c8a807466d8668";
pub const GALLERY_VIEW: &str = "eb18a135be254953a959999dfb3255c0";

/* Example Usage: Block Entity > TYPES > TEXT_BLOCK */
pub const TEXT_BLOCK: &str = "8426caa143d647d4a6f100c7c1a9a320";

/* Example Usage: Block Entity > TYPES > IMAGE_BLOCK */
pub const IMAGE_BLOCK: &str = "f0553d4d4838425ebcd7613bd8f475a5";

/* Example Usage: Entity > BLOCKS > Some_Entity_Of_Type_TEXT_BLOCK_or_TABLE_BLOCK */
pub const BLOCKS: &str = "beaba5cba67741a8b35377030613fc70";

/* Example Usage: Block Entity > PARENT_ENTITY > Some_Entity_ID */
pub const PARENT_ENTITY: &str = "dd4999b977f04c2ba02b5a26b233854e";

/* Example Usage:
Block Entity > TYPES > TEXT_BLOCK
Block Entity > MARKDOWN_CONTENT > "**hello world!**" */
pub const MARKDOWN_CONTENT: &str = "f88047cebd8d4fbf83f658e84ee533e4";

/* Example Usage:
Block Entity > TYPES > TABLE_BLOCK
Block Entity > ROW_TYPE > Some_Type_ID */
pub const ROW_TYPE: &str = "577bd9fbb29e4e2bb5f8f48aedbd26ac";

pub const FILTER: &str = "b0f2d71a79ca4dc49218e3e40dfed103";

pub const WALLETS_ATTRIBUTE: &str = "31f6922e0d4e4f14a1ee8c7689457715";

pub const PEOPLE_SPACE: &str = "0xb4476A42A66eC1356A58D300555169E17db6756c";

pub const BROADER_SPACES: &str = "03aa11edd69a4d5ea0aea0f197614cfd";

/**
 * Addresses for important contracts on our L3.
 *
 * Note: If you want to test deployments on a different network (e.g. local or Mumbai),
 * you can update these addresses to point to the correct contracts on that network.
 */

pub const PROFILE_REGISTRY_ADDRESS: &str = "0xc066E89bF7669b905f869Cb936818b0fd0bc456d";
pub const MEMBERSHIP_CONTRACT_ADDRESS: &str = "0x34a94160f4B0f86d932927DFfb326354dB279181";

// This is the address for the Root Space.
pub const ROOT_SPACE_ADDRESS: &str = "0xEcC4016C71fF38B32f01538207B6F0FdcbCF99f5";
pub const ROOT_SPACE_ID: &str = "ab7d4b9e02f840dab9746d352acb0ac6";

// This represents the beacon for the first set of deployed permissioned spaces.
// We should use this beacon for all new permissioned spaces. We need to track the beacon
// address in case we decide to upgrade the implementation of the permissionless space.
pub const PERMISSIONED_SPACE_BEACON_ADDRESS: &str = "0xe44Be15e413169Ad49fB24CBF8db192BE5A9A8bF";
// '0xf7239cb6d1ac800f2025a2571ce32bde190059cb' // mumbai

// This represents the Space contract acting as the registry for all permissioned spaces.
// This is the address for the Root Space.
pub const PERMISSIONED_SPACE_REGISTRY_ADDRESS: &str = "0x170b749413328ac9a94762031a7A05b00c1D2e34";

// This represents the beacon for all permissionless spaces. We need to track the beacon
// address in case we decide to upgrade the implementation of the permissionless space.
pub const PERMISSIONLESS_SPACE_BEACON_ADDRESS: &str = "0xf14C33B732851ECccA5e2c84a9b0DB6Eb24a5a4A";
// '0xc90513962Db42C1fb44fBb97a8eb0c2E102701Da' // mumbai

// This represents the PermissionlessSpace contract acting as the registry for all
// permissionless spaces.
pub const PERMISSIONLESS_SPACE_REGISTRY_ADDRESS: &str =
    "0x68930a23A91A8FA97C6053cD5057431BaD3eEB52";
// '0x42096035524630382E73cfFAE1CA319CFa72F4dC' // mumbai

// @TODO(migration)
// migrate types to new data model

// Root space
// pub const ROOT_SPACE = `0x170b749413328ac9a94762031a7A05b00c1D2e34` // @TODO(migration): update when we deploy new root space
// pub const ROOT_SPACE_CONFIGURATION = `f1b9fd886388436e95b551aafaea77e5`

// Page types
pub const POSTS_PAGE: &str = "e73c3db8320042309ae952eddb73b566";
pub const PRODUCTS_PAGE: &str = "6764f3827ff247e2b2ad295791153705";
pub const SERVICES_PAGE: &str = "e5d69a755ede4a56b43344e5d3fde7bc";
pub const EVENTS_PAGE: &str = "bb2917434c394223afba91a08aa83478";
pub const TEAM_PAGE: &str = "979eb04cefa942b6bd10229bf7f0ce21";
pub const JOBS_PAGE: &str = "abb4700856554b27bae8e7dba063b394";
pub const PROJECTS_PAGE: &str = "7171ce7a83b940a2abe2751a54c1c245";
pub const FINANCES_PAGE: &str = "f20af8deb57c472ab13d0247c46a8eeb";
pub const SPACES_PAGE: &str = "970e41c7196e42d3af0ecee755651d5b";

// Page templates
pub const COMPANY_SPACE_CONFIGURATION_TEMPLATE: &str = "8f5e618f781644cbb795300e8078bf15";
pub const COMPANY_POSTS_PAGE_TEMPLATE: &str = "90bd4735b2214059a5cd4f3215ab79d1";
pub const COMPANY_PRODUCTS_PAGE_TEMPLATE: &str = "6e9da70f357a4fc5b9d58de5840db16a";
pub const COMPANY_SERVICES_PAGE_TEMPLATE: &str = "d572b1248b5e40948c6c25e531fc8a33";
pub const COMPANY_EVENTS_PAGE_TEMPLATE: &str = "6885104d79ea4db2a64cc8e8512533ea";
pub const COMPANY_JOBS_PAGE_TEMPLATE: &str = "9a7528b37fb041c492c31650b70aae69";

pub const NONPROFIT_SPACE_CONFIGURATION_TEMPLATE: &str = "df388a8b27f54676b2376a59ca4a3e7";
pub const NONPROFIT_POSTS_PAGE_TEMPLATE: &str = "d370fe7af7784a5283984140cdc9bba";
pub const NONPROFIT_PROJECTS_PAGE_TEMPLATE: &str = "ddce09f82413449e973551e2998551b";
pub const NONPROFIT_FINANCES_PAGE_TEMPLATE: &str = "3be01e21822742e0bd40868957e3ede";

pub const PERSON_SPACE_CONFIGURATION_TEMPLATE: &str = "25d4b5bb2f3a4854a9fedf2f5f12b5e4";
pub const PERSON_POSTS_PAGE_TEMPLATE: &str = "026362d45d414b8db6ef8ed10ecd0d89";

// Entity templates
pub const TEMPLATE_ATTRIBUTE: &str = "babd29fb968147d08b58cdafc3890e12";

// ================================================================
// Indexer Space
// ================================================================
// System attributes
pub const CREATED_AT_TIMESTAMP: &str = "6302c2b3b29643028b191fabde2ef69b";
pub const CREATED_AT_BLOCK: &str = "9528b79b3ada42f6b657a4f8b24124d6";
pub const UPDATED_AT_TIMESTAMP: &str = "7e11f900f65345df92d978d45ba750b2";
pub const UPDATED_AT_BLOCK: &str = "d7bc4976184642fcb33a5ef29b573105";

/// Geo Account
pub const GEO_ACCOUNT: &str = "a53955b0b99d40b385ee95602f11ad8a";

// Space attributes
pub const SPACE_PLUGIN_ADDRESS: &str = "bacada4cfbc14da188acd7d60fa3c7c1";
pub const SPACE_VOTING_PLUGIN_ADDRESS: &str = "aac96a56c1e74d3497d4e1c7cfa31ece";
pub const SPACE_MEMBER_PLUGIN_ADDRESS: &str = "bba3a3fad8da4be98be74cd22bafeb47";
/// Type of space, e.g., Public or Personal
pub const SPACE_KIND: &str = "65da3fab6e1c48b7921a6a3260119b48";

/// GEO_ACCOUNT > MEMBER_RELATION > INDEXED_SPACE
pub const MEMBER_RELATION: &str = "fe2c4540550641f8aa92a67ee9490c95";

/// GEO_ACCOUNT > EDITOR_RELATION > INDEXED_SPACE
pub const EDITOR_RELATION: &str = "b5e0508bcc2943b7a31c2ac283c0c157";

/// SPACE > PARENT_SPACE > SPACE
pub const PARENT_SPACE: &str = "54ff7bba2ef74268acb4d0bc6c4b8725";

/// Special space for the indexer data
pub const INDEXER_SPACE_ID: &str = "bf03e5e1dbd7472c9c74fa7aac1545bc";

// Voting
/// GEO_ACCOUNT > VOTE_CAST > PROPOSAL
pub const VOTE_CAST: &str = "36c0262b99b34676ab733c32850bd2b1";

// Proposal
pub const PROPOSAL_TYPE: &str = "61055d985b9d42a19e15daab29f1248d";
pub const MEMBERSHIP_PROPOSAL_TYPE: &str = "06a84ba84cc1442cb3c1ef89507b5e87";
pub const EDITORSHIP_PROPOSAL_TYPE: &str = "50a93cea5c524e88915866d557f54f0d";
pub const SUBSPACE_PROPOSAL_TYPE: &str = "44d5a6d2d91945059ad12c2c577dd686";

/// MEMBERSHIP_PROPOSAL_TYPE > PROPOSED_ACCOUNT > GEO_ACCOUNT
/// EDITORSHIP_PROPOSAL_TYPE > PROPOSED_ACCOUNT > GEO_ACCOUNT
pub const PROPOSED_ACCOUNT: &str = "711f6bfcc009464fa274ac0e3a9f0382";

/// SUBSPACE_PROPOSAL_TYPE > PROPOSED_SUBSPACE > INDEXED_SPACE
pub const PROPOSED_SUBSPACE: &str = "53fd9bd36c9949be8981ee117c5019e1";

/// INDEXED_SPACE > PROPOSALS > PROPOSAL
pub const PROPOSALS: &str = "6bf8735530fa4f8bb6b024eb84f78ed0";
