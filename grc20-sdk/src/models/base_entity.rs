use grc20_core::system_ids;

#[grc20_core::entity]
pub struct BaseEntity {
    #[grc20(attribute = system_ids::NAME_ATTRIBUTE)]
    name: Option<String>,

    #[grc20(attribute = system_ids::DESCRIPTION_ATTRIBUTE)]
    description: Option<String>,
}