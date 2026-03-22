//! HG-PLUGIN — HypercubePlugin trait for custom extensions.

use crate::cell::{AttributeKey, AttributeKeyDef, DimKey, TypedAttrValue};
use crate::dim::DimensionAxis;
use crate::error::HypergridResult;

/// The extension interface for all Hypergrid customizations.
/// Plugins provide custom dimension types, attribute types, computed models,
/// rendering adapters, and data connectors.
pub trait HypercubePlugin: Send + Sync {
    fn plugin_id(&self) -> &str;
    fn name(&self) -> &str;

    /// Validate a key for a custom axis type.
    fn validate_key(&self, _axis: &DimensionAxis, _key: &DimKey) -> HypergridResult<()> {
        Ok(())
    }

    /// Provide additional attribute key definitions for registered cubes.
    fn attribute_key_defs(&self) -> Vec<AttributeKeyDef> {
        vec![]
    }

    /// Compute a Tier-2 AI/ML attribute value (async in production).
    fn compute_attribute(
        &self,
        _key: &AttributeKey,
        _context: &ComputeContext,
    ) -> HypergridResult<TypedAttrValue> {
        Ok(TypedAttrValue::Null)
    }
}

/// Context passed to plugin compute functions.
pub struct ComputeContext<'a> {
    pub cube_id: uuid::Uuid,
    pub coord: &'a crate::cell::DimCoordinate,
    pub current_attrs: &'a crate::cell::AttributeMap,
}

/// A no-op plugin — useful for testing.
pub struct NoOpPlugin {
    pub id: String,
}

impl HypercubePlugin for NoOpPlugin {
    fn plugin_id(&self) -> &str { &self.id }
    fn name(&self) -> &str { "no-op" }
}
