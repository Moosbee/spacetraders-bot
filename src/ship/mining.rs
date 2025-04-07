use chrono::{DateTime, Utc};
use space_traders_client::{
    apis::fleet_api::{CreateSurveyError, ExtractResourcesError, SiphonResourcesError},
    models::{self},
};

use super::MyShip;

impl MyShip {
    pub fn is_on_cooldown(&self) -> bool {
        if self.cooldown_expiration.is_some() {
            let t = self.cooldown_expiration.unwrap();
            let t = t - Utc::now();
            let t = t.num_seconds();
            t > 0
        } else {
            false
        }
    }

    pub async fn wait_for_cooldown_mut(
        &mut self,
        api: &space_traders_client::Api,
    ) -> crate::error::Result<()> {
        self.mutate();
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

    pub fn wait_for_cooldown(&self) -> impl std::future::Future<Output = ()> {
        if let Some(cooldown_expiration) = self.cooldown_expiration {
            let time_until_cooldown = cooldown_expiration.signed_duration_since(Utc::now());
            if time_until_cooldown.num_milliseconds() > 0 {
                return tokio::time::sleep(time_until_cooldown.to_std().unwrap());
            }
        }
        tokio::time::sleep(std::time::Duration::ZERO)
    }

    pub async fn extract(
        &mut self,
        api: &space_traders_client::Api,
    ) -> Result<
        models::ExtractResources201Response,
        space_traders_client::apis::Error<ExtractResourcesError>,
    > {
        self.mutate();
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
        api: &space_traders_client::Api,
        survey: &models::Survey,
    ) -> Result<
        models::ExtractResources201Response,
        space_traders_client::apis::Error<
            space_traders_client::apis::fleet_api::ExtractResourcesWithSurveyError,
        >,
    > {
        self.mutate();
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
        api: &space_traders_client::Api,
    ) -> Result<
        models::SiphonResources201Response,
        space_traders_client::apis::Error<SiphonResourcesError>,
    > {
        self.mutate();
        let extraction = api.siphon_resources(&self.symbol).await?;

        self.update_cooldown(&extraction.data.cooldown);
        self.cargo.update(&extraction.data.cargo);
        self.notify().await;

        Ok(extraction)
    }

    pub async fn survey(
        &mut self,
        api: &space_traders_client::Api,
    ) -> Result<models::CreateSurvey201Response, space_traders_client::apis::Error<CreateSurveyError>>
    {
        self.mutate();
        let survey = api.create_survey(&self.symbol).await?;

        self.update_cooldown(&survey.data.cooldown);
        self.notify().await;

        Ok(survey)
    }

    pub fn update_cooldown(&mut self, cooldown: &models::Cooldown) {
        self.mutate();
        let cool_text = cooldown.expiration.as_ref().map_or("", |v| v);
        self.cooldown_expiration = DateTime::parse_from_rfc3339(cool_text)
            .map(|op| op.to_utc())
            .ok();
    }
}
