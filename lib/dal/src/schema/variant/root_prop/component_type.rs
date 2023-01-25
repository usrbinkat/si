//! This module contains the ability to switch a [`Component's`](crate::Component) type between
//! a standard [`Component`](crate::Component) and a "frame". This functionality resides in this
//! location because it corresponds to the "/root/si/type" location in the
//! [`RootProp`](crate::RootProp) tree.

use serde::Deserialize;
use serde::Serialize;

/// The possible values of "/root/si/type".
#[derive(Deserialize, Serialize, Debug, Copy, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum ComponentType {
    Component,
    ConfigurationFrame,
    AggregationFrame,
}

impl ComponentType {
    /// Return the label corresponding to [`self`](Self).
    pub fn label(&self) -> &'static str {
        match self {
            Self::Component => "Component",
            Self::ConfigurationFrame => "Configuration Frame",
            Self::AggregationFrame => "Aggregation Frame",
        }
    }
}