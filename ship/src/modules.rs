use space_traders_client::models;

#[derive(Debug, Default, serde::Serialize, Clone, async_graphql::SimpleObject)]
#[graphql(complex)]
pub struct ModuleState {
    pub modules: Vec<models::ship_module::Symbol>,
}

impl ModuleState {
    pub fn update(&mut self, modules: &[models::ShipModule]) {
        self.modules = modules.iter().map(|m| m.symbol).collect();
    }

    pub fn can_cargo(&self) -> bool {
        self.modules.iter().any(|m| {
            m == &models::ship_module::Symbol::CargoHoldI
                || m == &models::ship_module::Symbol::CargoHoldIi
                || m == &models::ship_module::Symbol::CargoHoldIii
        })
    }
}

#[async_graphql::ComplexObject]
impl ModuleState {
    async fn module_infos<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> crate::Result<Vec<database::ModuleInfo>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let mut modules = Vec::new();
        for module_symbol in self.modules.iter() {
            let erg = database::ModuleInfo::get_by_id(database_pool, module_symbol).await?;
            modules.push(erg);
        }
        Ok(modules)
    }
}
