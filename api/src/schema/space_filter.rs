use juniper::GraphQLInputObject;
use grc20_core::mapping::query_utils::PropFilter;

use super::space::SpaceGovernanceType;

#[derive(Debug, GraphQLInputObject)]
pub struct SpaceFilter {
    pub id: Option<String>,
    pub id_not: Option<String>,
    pub id_in: Option<Vec<String>>,
    pub id_not_in: Option<Vec<String>>,

    pub network: Option<String>,
    pub network_not: Option<String>,
    pub network_in: Option<Vec<String>>,
    pub network_not_in: Option<Vec<String>>,

    pub governance_type: Option<SpaceGovernanceType>,
    pub governance_type_not: Option<SpaceGovernanceType>,
    pub governance_type_in: Option<Vec<SpaceGovernanceType>>,
    pub governance_type_not_in: Option<Vec<SpaceGovernanceType>>,

    pub dao_contract_address: Option<String>,
    pub dao_contract_address_not: Option<String>,
    pub dao_contract_address_in: Option<Vec<String>>,
    pub dao_contract_address_not_in: Option<Vec<String>>,

    pub space_plugin_address: Option<String>,
    pub space_plugin_address_not: Option<String>,
    pub space_plugin_address_in: Option<Vec<String>>,
    pub space_plugin_address_not_in: Option<Vec<String>>,

    pub voting_plugin_address: Option<String>,
    pub voting_plugin_address_not: Option<String>,
    pub voting_plugin_address_in: Option<Vec<String>>,
    pub voting_plugin_address_not_in: Option<Vec<String>>,

    pub member_access_plugin: Option<String>,
    pub member_access_plugin_not: Option<String>,
    pub member_access_plugin_in: Option<Vec<String>>,
    pub member_access_plugin_not_in: Option<Vec<String>>,

    pub personal_space_admin_plugin: Option<String>,
    pub personal_space_admin_plugin_not: Option<String>,
    pub personal_space_admin_plugin_in: Option<Vec<String>>,
    pub personal_space_admin_plugin_not_in: Option<Vec<String>>,
}

impl SpaceFilter {
    pub fn id_filter(&self) -> PropFilter<String> {
        let mut filter = PropFilter::default();

        if let Some(id) = &self.id {
            filter = filter.value(id);
        }

        if let Some(id_not) = &self.id_not {
            filter = filter.value_not(id_not);
        }

        if let Some(id_in) = &self.id_in {
            filter = filter.value_in(id_in.clone());
        }

        if let Some(id_not_in) = &self.id_not_in {
            filter = filter.value_not_in(id_not_in.clone());
        }

        filter
    }

    pub fn network_filter(&self) -> Option<PropFilter<String>> {
        if self.network.is_none()
            && self.network_not.is_none()
            && self.network_in.is_none()
            && self.network_not_in.is_none()
        {
            return None;
        }

        let mut filter = PropFilter::default();

        if let Some(network) = &self.network {
            filter = filter.value(network);
        }

        if let Some(network_not) = &self.network_not {
            filter = filter.value_not(network_not);
        }

        if let Some(network_in) = &self.network_in {
            filter = filter.value_in(network_in.clone());
        }

        if let Some(network_not_in) = &self.network_not_in {
            filter = filter.value_not_in(network_not_in.clone());
        }

        Some(filter)
    }

    pub fn governance_type_filter(&self) -> Option<PropFilter<grc20_sdk::models::space::SpaceGovernanceType>> {
        if self.governance_type.is_none()
            && self.governance_type_not.is_none()
            && self.governance_type_in.is_none()
            && self.governance_type_not_in.is_none()
        {
            return None;
        }

        let mut filter = PropFilter::default();

        if let Some(governance_type) = &self.governance_type {
            filter = filter.value(governance_type);
        }

        if let Some(governance_type_not) = &self.governance_type_not {
            filter = filter.value_not(governance_type_not);
        }

        if let Some(governance_type_in) = &self.governance_type_in {
            filter = filter.value_in(governance_type_in.into_iter().map(|g| g.into()).collect());
        }

        if let Some(governance_type_not_in) = &self.governance_type_not_in {
            filter = filter.value_not_in(governance_type_not_in.into_iter().map(|g| g.into()).collect());
        }

        Some(filter)
    }

    pub fn dao_contract_address_filter(&self) -> Option<PropFilter<String>> {
        if self.dao_contract_address.is_none()
            && self.dao_contract_address_not.is_none()
            && self.dao_contract_address_in.is_none()
            && self.dao_contract_address_not_in.is_none()
        {
            return None;
        }

        let mut filter = PropFilter::default();

        if let Some(dao_contract_address) = &self.dao_contract_address {
            filter = filter.value(dao_contract_address);
        }

        if let Some(dao_contract_address_not) = &self.dao_contract_address_not {
            filter = filter.value_not(dao_contract_address_not);
        }

        if let Some(dao_contract_address_in) = &self.dao_contract_address_in {
            filter = filter.value_in(dao_contract_address_in.clone());
        }

        if let Some(dao_contract_address_not_in) = &self.dao_contract_address_not_in {
            filter = filter.value_not_in(dao_contract_address_not_in.clone());
        }

        Some(filter)
    }

    pub fn space_plugin_address_filter(&self) -> Option<PropFilter<String>> {
        if self.space_plugin_address.is_none()
            && self.space_plugin_address_not.is_none()
            && self.space_plugin_address_in.is_none()
            && self.space_plugin_address_not_in.is_none()
        {
            return None;
        }

        let mut filter = PropFilter::default();

        if let Some(space_plugin_address) = &self.space_plugin_address {
            filter = filter.value(space_plugin_address);
        }

        if let Some(space_plugin_address_not) = &self.space_plugin_address_not {
            filter = filter.value_not(space_plugin_address_not);
        }

        if let Some(space_plugin_address_in) = &self.space_plugin_address_in {
            filter = filter.value_in(space_plugin_address_in.clone());
        }

        if let Some(space_plugin_address_not_in) = &self.space_plugin_address_not_in {
            filter = filter.value_not_in(space_plugin_address_not_in.clone());
        }

        Some(filter)
    }

    pub fn voting_plugin_address_filter(&self) -> Option<PropFilter<String>> {
        if self.voting_plugin_address.is_none()
            && self.voting_plugin_address_not.is_none()
            && self.voting_plugin_address_in.is_none()
            && self.voting_plugin_address_not_in.is_none()
        {
            return None;
        }

        let mut filter = PropFilter::default();

        if let Some(voting_plugin_address) = &self.voting_plugin_address {
            filter = filter.value(voting_plugin_address);
        }

        if let Some(voting_plugin_address_not) = &self.voting_plugin_address_not {
            filter = filter.value_not(voting_plugin_address_not);
        }

        if let Some(voting_plugin_address_in) = &self.voting_plugin_address_in {
            filter = filter.value_in(voting_plugin_address_in.clone());
        }

        if let Some(voting_plugin_address_not_in) = &self.voting_plugin_address_not_in {
            filter = filter.value_not_in(voting_plugin_address_not_in.clone());
        }

        Some(filter)
    }

    pub fn member_access_plugin_filter(&self) -> Option<PropFilter<String>> {
        if self.member_access_plugin.is_none()
            && self.member_access_plugin_not.is_none()
            && self.member_access_plugin_in.is_none()
            && self.member_access_plugin_not_in.is_none()
        {
            return None;
        }

        let mut filter = PropFilter::default();

        if let Some(member_access_plugin) = &self.member_access_plugin {
            filter = filter.value(member_access_plugin);
        }

        if let Some(member_access_plugin_not) = &self.member_access_plugin_not {
            filter = filter.value_not(member_access_plugin_not);
        }

        if let Some(member_access_plugin_in) = &self.member_access_plugin_in {
            filter = filter.value_in(member_access_plugin_in.clone());
        }

        if let Some(member_access_plugin_not_in) = &self.member_access_plugin_not_in {
            filter = filter.value_not_in(member_access_plugin_not_in.clone());
        }

        Some(filter)
    }

    pub fn personal_space_admin_plugin_filter(&self) -> Option<PropFilter<String>> {
        if self.personal_space_admin_plugin.is_none()
            && self.personal_space_admin_plugin_not.is_none()
            && self.personal_space_admin_plugin_in.is_none()
            && self.personal_space_admin_plugin_not_in.is_none()
        {
            return None;
        }

        let mut filter = PropFilter::default();

        if let Some(personal_space_admin_plugin) = &self.personal_space_admin_plugin {
            filter = filter.value(personal_space_admin_plugin);
        }

        if let Some(personal_space_admin_plugin_not) = &self.personal_space_admin_plugin_not {
            filter = filter.value_not(personal_space_admin_plugin_not);
        }

        if let Some(personal_space_admin_plugin_in) = &self.personal_space_admin_plugin_in {
            filter = filter.value_in(personal_space_admin_plugin_in.clone());
        }

        if let Some(personal_space_admin_plugin_not_in) = &self.personal_space_admin_plugin_not_in {
            filter = filter.value_not_in(personal_space_admin_plugin_not_in.clone());
        }

        Some(filter)
    }
}
