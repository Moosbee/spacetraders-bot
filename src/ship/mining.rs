use chrono::{DateTime, Utc};
use space_traders_client::{
    apis::fleet_api::{CreateSurveyError, ExtractResourcesError, SiphonResourcesError},
    models::{self},
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

    pub async fn wait_for_cooldown(&mut self, api: &api::Api) -> anyhow::Result<()> {
        if self.cooldown_expiration.is_none() {
            return Ok(());
        }
        let t = self.cooldown_expiration.unwrap();
        let t = t - Utc::now();
        let t = t.num_seconds().try_into();
        if let Ok(t) = t {
            self.sleep(std::time::Duration::from_secs(t), api).await;
        } else {
            self.try_recive_update(api).await;
        }
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
        self.notify().await;

        Ok(extraction)
    }

    pub async fn extract_with_survey(
        &mut self,
        api: &api::Api,
        survey: &models::Survey,
    ) -> Result<
        models::ExtractResources201Response,
        space_traders_client::apis::Error<
            space_traders_client::apis::fleet_api::ExtractResourcesWithSurveyError,
        >,
    > {
        let extraction = api
            .extract_resources_with_survey(&self.symbol, Some(survey.clone()))
            .await?;

        self.update_cooldown(&extraction.data.cooldown);
        self.cargo.update(&extraction.data.cargo);
        self.notify().await;

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
        self.notify().await;

        Ok(extraction)
    }

    pub async fn survey(
        &mut self,
        api: &api::Api,
    ) -> Result<models::CreateSurvey201Response, space_traders_client::apis::Error<CreateSurveyError>>
    {
        let survey = api.create_survey(&self.symbol).await?;

        self.update_cooldown(&survey.data.cooldown);
        self.notify().await;

        Ok(survey)
    }

    pub fn update_cooldown(&mut self, cooldown: &models::Cooldown) {
        let cool_text = cooldown.expiration.as_ref().map_or("", |v| v);
        self.cooldown_expiration = DateTime::parse_from_rfc3339(&cool_text)
            .map(|op| op.to_utc())
            .ok();
    }
}
