use space_traders_client::models;

#[derive(Debug, Default, serde::Serialize, Clone, async_graphql::SimpleObject)]
#[graphql(complex)]
pub struct MountState {
    pub mounts: Vec<models::ship_mount::Symbol>,
}

impl MountState {
    pub fn update(&mut self, mounts: &[models::ShipMount]) {
        self.mounts = mounts.iter().map(|m| m.symbol).collect();
    }

    pub fn can_extract(&self) -> bool {
        self.mounts.iter().any(|m| {
            m == &models::ship_mount::Symbol::MiningLaserI
                || m == &models::ship_mount::Symbol::MiningLaserIi
                || m == &models::ship_mount::Symbol::MiningLaserIii
        })
    }

    pub fn can_siphon(&self) -> bool {
        self.mounts.iter().any(|m| {
            m == &models::ship_mount::Symbol::GasSiphonI
                || m == &models::ship_mount::Symbol::GasSiphonIi
                || m == &models::ship_mount::Symbol::GasSiphonIii
        })
    }

    pub fn can_survey(&self) -> bool {
        self.mounts.iter().any(|m| {
            m == &models::ship_mount::Symbol::SurveyorI
                || m == &models::ship_mount::Symbol::SurveyorIi
                || m == &models::ship_mount::Symbol::SurveyorIii
        })
    }

    pub fn can_scan(&self) -> bool {
        self.mounts.iter().any(|m| {
            m == &models::ship_mount::Symbol::SensorArrayI
                || m == &models::ship_mount::Symbol::SensorArrayIi
                || m == &models::ship_mount::Symbol::SensorArrayIii
        })
    }
}

#[async_graphql::ComplexObject]
impl MountState {
    async fn mount_infos<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> crate::Result<Vec<database::MountInfo>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let mut mounts = Vec::new();
        for mount_symbol in self.mounts.iter() {
            let erg = database::MountInfo::get_by_id(database_pool, mount_symbol).await?;
            mounts.push(erg);
        }
        Ok(mounts)
    }
}
