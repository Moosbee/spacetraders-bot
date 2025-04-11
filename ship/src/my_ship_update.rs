use futures::FutureExt;
use log::debug;
use space_traders_client::models::{self, TradeSymbol};
use tokio::{select, sync::broadcast};
use utils::SendFuture;

use crate::MyShip;

#[derive(Debug)]
pub struct InterShipBroadcaster {
    pub sender: tokio::sync::broadcast::Sender<MyShipUpdate>,
    pub receiver: tokio::sync::broadcast::Receiver<MyShipUpdate>,
}

impl Default for InterShipBroadcaster {
    fn default() -> Self {
        let (sender, receiver) = broadcast::channel(16);
        Self { sender, receiver }
    }
}

impl Clone for InterShipBroadcaster {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
            receiver: self.receiver.resubscribe(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct MyShipUpdate {
    pub symbol: String,
    pub update: ShipUpdate,
}

#[derive(Debug, Clone, Default)]
pub enum ShipUpdate {
    CargoChange(CargoChange),
    TransferRequest(TransferRequest),
    #[default]
    None,
}

#[derive(Debug, Clone, Default)]
pub struct CargoChange {
    pub units: i32,
    pub trade_symbol: TradeSymbol,
}

#[derive(Debug, Clone)]
pub struct TransferRequest {
    pub units: i32,
    pub trade_symbol: TradeSymbol,
    pub target: String,
    pub callback: tokio::sync::mpsc::Sender<()>,
}

impl MyShip {
    pub fn from_ship(ship: models::Ship, broadcaster: InterShipBroadcaster) -> MyShip {
        let mut new_ship = MyShip::default();
        new_ship.update(ship);
        new_ship.broadcaster = broadcaster;
        new_ship
    }

    pub async fn try_recive_update(&mut self, api: &space_traders_client::Api) {
        self.mutate();
        while let Ok(data) = self.broadcaster.receiver.try_recv() {
            self.handle_update(data, api).await;
        }
    }

    async fn receive_update_loop(
        &mut self,
        cancel: &tokio_util::sync::CancellationToken,
        api: &space_traders_client::Api,
    ) {
        loop {
            let data = select! {
                data = self.broadcaster.receiver.recv() => data,
                _ = cancel.cancelled() => break,
            };
            if let Ok(data) = data {
                self.handle_update(data, api).await;
            }
            if cancel.is_cancelled() {
                break;
            }
        }
    }

    async fn handle_update(&mut self, data: MyShipUpdate, api: &space_traders_client::Api) {
        if data.symbol != self.symbol {
            return;
        }
        debug!(
            "Handling update: {:?} for ship: {}",
            data.update, self.symbol
        );
        let erg: std::result::Result<(), crate::error::Error> = match data.update {
            ShipUpdate::CargoChange(cargo_change) => self
                .cargo
                .handle_cago_update(cargo_change.units, cargo_change.trade_symbol),
            ShipUpdate::TransferRequest(transfer_request) => {
                let erg = self
                    .transfer_cargo(
                        transfer_request.trade_symbol,
                        transfer_request.units,
                        api,
                        &transfer_request.target,
                    )
                    .await;
                debug!("Transfer cargo: {:?} {}", erg, self.symbol);
                let _reg: std::result::Result<(), tokio::sync::mpsc::error::SendError<()>> =
                    transfer_request.callback.send(()).await;
                erg.map(|_| ())
            }
            ShipUpdate::None => Ok(()),
        };
        if let Err(e) = erg {
            log::error!("Failed to handle update: {}", e);
        }
        self.notify().await;
    }

    #[deprecated]
    pub async fn sleep(&mut self, duration: std::time::Duration, api: &space_traders_client::Api) {
        self.mutate();
        let cancel = tokio_util::sync::CancellationToken::new();

        let update_future = self.receive_update_loop(&cancel, api);
        let sleep_future = tokio::time::sleep(duration).then(|_| {
            cancel.cancel();

            async move {}
        });
        let _erg = futures::future::join(update_future, sleep_future)
            .send() // https://github.com/rust-lang/rust/issues/96865
            .await;
    }

    pub async fn transfer_cargo(
        &mut self,
        trade_symbol: space_traders_client::models::TradeSymbol,
        units: i32,
        api: &space_traders_client::Api,
        target_ship: &str,
    ) -> crate::error::Result<space_traders_client::models::TransferCargo200Response> {
        self.mutate();

        let transfer_result = self
            .simple_transfer_cargo(trade_symbol, units, api, target_ship)
            .await?;

        let update_event = MyShipUpdate {
            symbol: target_ship.to_string(),
            update: ShipUpdate::CargoChange(CargoChange {
                trade_symbol,
                units,
            }),
        };
        debug!("Sending update event: {:#?}", update_event);
        self.broadcaster
            .sender
            .send(update_event)
            .map_err(|err| crate::error::Error::General(err.to_string()))?;

        Ok(transfer_result)
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
}
