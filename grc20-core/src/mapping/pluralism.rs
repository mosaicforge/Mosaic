use super::{aggregation::SpaceRanking, AggregationDirection};

pub enum Pluralism {
    None,
    Direction(AggregationDirection),
    Hierarchy(Vec<SpaceRanking>),
}
