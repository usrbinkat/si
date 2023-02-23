use serde::{Deserialize, Serialize};
use si_data_nats::NatsError;
use si_data_pg::PgError;
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    impl_standard_model, pk, standard_model, standard_model_accessor, DalContext,
    HistoryEventError, StandardModel, StandardModelError, Tenancy, Timestamp, Visibility,
};

#[derive(Error, Debug)]
pub enum InstalledPkgError {
    #[error("error serializing/deserializing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("nats txn error: {0}")]
    Nats(#[from] NatsError),
    #[error("history event error: {0}")]
    HistoryEvent(#[from] HistoryEventError),
    #[error("standard model error: {0}")]
    StandardModelError(#[from] StandardModelError),
    #[error("error decoding code_base64: {0}")]
    Decode(#[from] base64::DecodeError),
}

pub type InstalledPkgResult<T> = Result<T, InstalledPkgError>;

pk!(InstalledPkgPk);
pk!(InstalledPkgId);

/// An `InstalledPkg` is a record of the installation of a package. It tracks the
/// package installation and can be used to prevent duplicate installations and
/// to remove packages after installation.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct InstalledPkg {
    pk: InstalledPkgPk,
    id: InstalledPkgId,
    name: String,
    root_hash: Option<String>,    // this will become non-optional
    pkg_contents: Option<String>, // same
    #[serde(flatten)]
    tenancy: Tenancy,
    #[serde(flatten)]
    timestamp: Timestamp,
    #[serde(flatten)]
    visibility: Visibility,
}

impl_standard_model! {
    model: InstalledPkg,
    pk: InstalledPkgPk,
    id: InstalledPkgId,
    table_name: "installed_pkgs",
    history_event_label_base: "installed_pkg",
    history_event_message_name: "Installed Pkg"
}

impl InstalledPkg {
    #[instrument(skip_all)]
    pub async fn new(
        ctx: &DalContext,
        name: impl AsRef<str>,
        root_hash: Option<String>,
        pkg_contents: Option<String>,
    ) -> InstalledPkgResult<Self> {
        let name = name.as_ref();
        let row = ctx
            .txns()
            .pg()
            .query_one(
                "SELECT object FROM installed_pkg_create_v1($1, $2, $3, $4, $5)",
                &[
                    ctx.tenancy(),
                    ctx.visibility(),
                    &name,
                    &root_hash,
                    &pkg_contents,
                ],
            )
            .await?;
        let object = standard_model::finish_create_from_row(ctx, row).await?;
        Ok(object)
    }

    standard_model_accessor!(name, String, InstalledPkgResult);
    standard_model_accessor!(root_hash, Option<String>, InstalledPkgResult);
    standard_model_accessor!(pkg_contents, Option<String>, InstalledPkgResult);
}
