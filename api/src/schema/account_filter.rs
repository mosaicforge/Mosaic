use grc20_core::mapping::query_utils::PropFilter;
use juniper::GraphQLInputObject;

#[derive(Debug, GraphQLInputObject)]
pub struct AccountFilter {
    pub id: Option<String>,
    pub id_not: Option<String>,
    pub id_in: Option<Vec<String>>,
    pub id_not_in: Option<Vec<String>>,

    pub address: Option<String>,
    pub address_not: Option<String>,
    pub address_in: Option<Vec<String>>,
    pub address_not_in: Option<Vec<String>>,
}

impl AccountFilter {
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

    pub fn address_filter(&self) -> Option<PropFilter<String>> {
        if self.address.is_none()
            && self.address_not.is_none()
            && self.address_in.is_none()
            && self.address_not_in.is_none()
        {
            return None;
        }

        let mut filter = PropFilter::default();

        if let Some(address) = &self.address {
            filter = filter.value(address);
        }

        if let Some(address_not) = &self.address_not {
            filter = filter.value_not(address_not);
        }

        if let Some(address_in) = &self.address_in {
            filter = filter.value_in(address_in.clone());
        }

        if let Some(address_not_in) = &self.address_not_in {
            filter = filter.value_not_in(address_not_in.clone());
        }

        Some(filter)
    }
}
