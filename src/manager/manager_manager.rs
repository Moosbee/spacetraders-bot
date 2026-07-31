use crate::{
    control_api,
    manager::{
        self, chart_manager::ChartManager, construction_manager::ConstructionManager,
        contract_manager::ContractManager, fleet_manager::FleetManager,
        mining_manager::MiningManager, scrapping_manager::ScrappingManager,
        ship_task::ShipTaskHandler, trade_manager::TradeManager,
    },
};

pub struct ManagerManager {
    construction_manager: ConstructionManager,
    contract_manager: ContractManager,
    mining_manager: MiningManager,
    scrapping_manager: ScrappingManager,
    trade_manager: TradeManager,
    fleet_manager: FleetManager,
    chart_manager: ChartManager,
    ship_task_handler: ShipTaskHandler,
    control_api: control_api::server::ControlApiServer,
}

impl ManagerManager {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        construction_manager: ConstructionManager,
        contract_manager: ContractManager,
        mining_manager: MiningManager,
        scrapping_manager: ScrappingManager,
        trade_manager: TradeManager,
        fleet_manager: FleetManager,
        chart_manager: ChartManager,
        ship_task_handler: ShipTaskHandler,
        control_api: control_api::server::ControlApiServer,
    ) -> Self {
        ManagerManager {
            construction_manager,
            contract_manager,
            mining_manager,
            scrapping_manager,
            trade_manager,
            fleet_manager,
            chart_manager,
            ship_task_handler,
            control_api,
        }
    }

    pub fn start(self) -> ManagerHandels {
        ManagerHandels {
            construction_manager: ManagersHandle::init(self.construction_manager),
            contract_manager: ManagersHandle::init(self.contract_manager),
            mining_manager: ManagersHandle::init(self.mining_manager),
            scrapping_manager: ManagersHandle::init(self.scrapping_manager),
            trade_manager: ManagersHandle::init(self.trade_manager),
            fleet_manager: ManagersHandle::init(self.fleet_manager),
            chart_manager: ManagersHandle::init(self.chart_manager),
            ship_task_handler: ManagersHandle::init(self.ship_task_handler),
            control_api: ManagersHandle::init(self.control_api),
        }
    }
}

struct ManagersHandle<T: manager::Manager> {
    handle: tokio::task::JoinHandle<(T, Result<(), crate::error::Error>)>,
    manager_name: String,
}

impl<T: manager::Manager + 'static> ManagersHandle<T> {
    pub fn init(mut manager: T) -> Self {
        let name = manager.get_name().to_string();
        let manager_name = name.clone();
        tracing::debug!(manager_name = %name, "Starting manager");
        let handle = utils::task_spawn(format!("manager-{}", manager_name).as_str(), async move {
            let erg = manager.run().await;

            if erg.is_ok() {
                tracing::info!(manager_name = %manager_name, erg = ?erg, "Manager finished and joined")
            } else {
                tracing::error!(manager_name = %manager_name, error = ?erg, "Manager error occurred");
            }

            (manager, erg)
        });
        Self {
            handle,
            manager_name: name,
        }
    }

    pub async fn wait(
        self,
        global_cancel_token: &tokio_util::sync::CancellationToken,
    ) -> Option<(T, Result<(), crate::error::Error>)> {
        let erg: Result<(T, Result<(), crate::error::Error>), tokio::task::JoinError> =
            self.handle.await;
        match erg {
            Ok(result) => Some(result),
            Err(e) => {
                tracing::error!(manager_name = %self.manager_name, error = ?e, "Manager error occurred");
                global_cancel_token.cancel();
                None
            }
        }
    }
}

pub struct ManagerHandels {
    construction_manager: ManagersHandle<ConstructionManager>,
    contract_manager: ManagersHandle<ContractManager>,
    mining_manager: ManagersHandle<MiningManager>,
    scrapping_manager: ManagersHandle<ScrappingManager>,
    trade_manager: ManagersHandle<TradeManager>,
    fleet_manager: ManagersHandle<FleetManager>,
    chart_manager: ManagersHandle<ChartManager>,
    ship_task_handler: ManagersHandle<ShipTaskHandler>,
    control_api: ManagersHandle<control_api::server::ControlApiServer>,
}

impl ManagerHandels {
    pub async fn wait(
        self,
        global_cancel_token: &tokio_util::sync::CancellationToken,
    ) -> Result<ManagerManager, anyhow::Error> {
        let erg = tokio::join!(
            self.construction_manager.wait(global_cancel_token),
            self.contract_manager.wait(global_cancel_token),
            self.mining_manager.wait(global_cancel_token),
            self.scrapping_manager.wait(global_cancel_token),
            self.trade_manager.wait(global_cancel_token),
            self.fleet_manager.wait(global_cancel_token),
            self.chart_manager.wait(global_cancel_token),
            self.ship_task_handler.wait(global_cancel_token),
            self.control_api.wait(global_cancel_token),
        );

        if let (
            Some(construction_manager),
            Some(contract_manager),
            Some(mining_manager),
            Some(scrapping_manager),
            Some(trade_manager),
            Some(fleet_manager),
            Some(chart_manager),
            Some(ship_task_handler),
            Some(control_api),
        ) = erg
        {
            Ok(ManagerManager::new(
                construction_manager.0,
                contract_manager.0,
                mining_manager.0,
                scrapping_manager.0,
                trade_manager.0,
                fleet_manager.0,
                chart_manager.0,
                ship_task_handler.0,
                control_api.0,
            ))
        } else {
            Err(anyhow::anyhow!("Manager error occurred"))
        }
    }
}
