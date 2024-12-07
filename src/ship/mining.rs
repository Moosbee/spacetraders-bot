use chrono::{DateTime, Utc};
use space_traders_client::{
    apis::fleet_api::{ExtractResourcesError, SiphonResourcesError},
    models,
};

use crate::api;

use super::MyShip;

impl MyShip {
    pub fn is_on_cooldown(&self) -> bool {
        if self.cooldown_expiration.is_some() {
            let t = self.cooldown_expiration.unwrap();
            let t = t - Utc::now();
            let t = t.num_seconds();
            t > 0
        } else {
            true
        }
    }

    pub async fn wait_for_cooldown(&self) -> anyhow::Result<()> {
        if self.cooldown_expiration.is_none() {
            return Err(anyhow::anyhow!("Is not on cooldown"));
        }
        let t = self.cooldown_expiration.unwrap();
        let t = t - Utc::now();
        let t = t.num_seconds().try_into()?;
        tokio::time::sleep(std::time::Duration::from_secs(t)).await;
        Ok(())
    }

    pub async fn extract(
        &mut self,
        api: &api::Api,
    ) -> Result<
        models::ExtractResources201Response,
        space_traders_client::apis::Error<ExtractResourcesError>,
    > {
        let extraction = api
            .extract_resources(&self.symbol, Some(Default::default()))
            .await?;

        self.update_cooldown(&extraction.data.cooldown);
        self.cargo.update(&extraction.data.cargo);

        Ok(extraction)
    }

    pub async fn siphon(
        &mut self,
        api: &api::Api,
    ) -> Result<
        models::SiphonResources201Response,
        space_traders_client::apis::Error<SiphonResourcesError>,
    > {
        let extraction = api.siphon_resources(&self.symbol).await?;

        self.update_cooldown(&extraction.data.cooldown);
        self.cargo.update(&extraction.data.cargo);

        Ok(extraction)
    }

    pub fn update_cooldown(&mut self, cooldown: &models::Cooldown) {
        let cool_text = cooldown.expiration.as_ref().map_or("", |v| v);
        self.cooldown_expiration = DateTime::parse_from_rfc3339(&cool_text)
            .map(|op| op.to_utc())
            .ok();
    }
}
