use tokio::{select, task::JoinSet};
use tracing::instrument;

use crate::utils::ConductorContext;

use super::Manager;

pub type ShipTaskHandlerReceiver = tokio::sync::mpsc::Receiver<database::ShipInfo>;

pub struct ShipTaskHandler {
    receiver: ShipTaskHandlerReceiver,
    slow_ship_cancel_token: tokio_util::sync::CancellationToken,
    fast_ship_cancel_token: tokio_util::sync::CancellationToken,
    slow_manager_cancel_token: tokio_util::sync::CancellationToken,
    slow_cancel_token: tokio_util::sync::CancellationToken,
    fast_cancel_token: tokio_util::sync::CancellationToken,
    context: ConductorContext,
}

type ShipFuture = ();

#[derive(Debug, Clone)]
pub struct ShipTaskMessanger {
    sender: tokio::sync::mpsc::Sender<database::ShipInfo>,
}

impl ShipTaskMessanger {
    pub async fn start_ship(&self, ship_names: database::ShipInfo) {
        tracing::debug!(ship_names = ?ship_names, "Starting ship");
        let _erg = self.sender.send(ship_names).await;
    }
}

impl ShipTaskHandler {
    pub fn create() -> (ShipTaskHandlerReceiver, ShipTaskMessanger) {
        let (sender, receiver) = tokio::sync::mpsc::channel(1024); // may become a problem if we have to many ships, set to 8192 if needed
        (receiver, ShipTaskMessanger { sender })
    }
    pub fn new(
        fast_ship_cancel_token: tokio_util::sync::CancellationToken,
        slow_ship_cancel_token: tokio_util::sync::CancellationToken,
        fast_cancel_token: tokio_util::sync::CancellationToken,
        slow_cancel_token: tokio_util::sync::CancellationToken,
        slow_manager_cancel_token: tokio_util::sync::CancellationToken,
        context: ConductorContext,
        receiver: ShipTaskHandlerReceiver,
    ) -> Self {
        Self {
            fast_cancel_token,
            fast_ship_cancel_token,
            slow_ship_cancel_token,
            receiver,
            slow_manager_cancel_token,
            slow_cancel_token,
            context,
        }
    }

    #[instrument(
        level = "info",
        name = "spacetraders::manager::ship_task_handler",
        skip(self)
    )]
    pub async fn await_all(&mut self) -> Result<(), crate::error::Error> {
        let mut set: JoinSet<(String, Result<ShipFuture, crate::error::Error>)> = JoinSet::new();

        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        tracing::info!("Starting ship_task await_all");

        while let Ok(ship_name) = self.receiver.try_recv() {
            tracing::debug!(ship_name = ?ship_name, "Starting initial ship task in await_all");
            let mut pilot = crate::pilot::Pilot::new(
                self.context.clone(),
                ship_name.symbol.clone(),
                self.fast_ship_cancel_token.child_token(),
                self.slow_ship_cancel_token.child_token(),
            );

            utils::task_spawn_set(
                &mut set,
                format!("ship-as-{}", ship_name.symbol).as_str(),
                async move { (ship_name.symbol.clone(), pilot.pilot_ship().await) },
            );
        }

        tracing::debug!("Starting ship_task await_all loop");

        loop {
            select! {
                ship_name = self.receiver.recv() => {
                    match ship_name {
                        Some(ship_name) => {
                            self.handle_ship_add(&mut set, ship_name).await?;
                        }
                        None => {
                            tracing::info!("ShipTaskHandler::await_all: receiver is closed");
                            break;
                        }
                    }
                }
                finished_future = set.join_next() => {
                  match finished_future {
                    Some(finished_future) => {
                        self.handle_finished_future(finished_future).await;
                    },
                    None => {
                        tracing::debug!("No finished future in await_all");
                        break;
                    }
                  }
                }
                _ = self.fast_cancel_token.cancelled() => {
                    tracing::info!("ShipTaskHandler::await_all: fast cancel token");
                    break;
                }
                // _ = self.slow_cancel_token.cancelled() => {
                //     tracing::info!("ShipTaskHandler::await_all: slow cancel token");
                //     break;
                // }
            }
        }

        self.slow_ship_cancel_token.cancel();
        self.slow_manager_cancel_token.cancel();
        Ok(())
    }

    async fn handle_ship_add(
        &mut self,
        set: &mut JoinSet<(String, Result<ShipFuture, crate::error::Error>)>,
        ship_name: database::ShipInfo,
    ) -> Result<(), crate::error::Error> {
        tracing::debug!(ship_name = ?ship_name, "Starting new ship task in await_all");
        let mut pilot = crate::pilot::Pilot::new(
            self.context.clone(),
            ship_name.symbol.clone(),
            self.fast_ship_cancel_token.child_token(),
            self.slow_ship_cancel_token.child_token(),
        );

        utils::task_spawn_set(
            set,
            format!("ship-as-{}", ship_name.symbol).as_str(),
            async move { (ship_name.symbol.clone(), pilot.pilot_ship().await) },
        );

        Ok(())
    }

    async fn handle_finished_future(
        &mut self,
        finished_future: Result<
            (String, Result<ShipFuture, crate::error::Error>),
            tokio::task::JoinError,
        >,
    ) {
        match finished_future {
            Ok((ship_name, Ok(erg))) => {
                tracing::debug!(ship_name = %ship_name, erg = ?erg, "Finished ship in await_all");
            }
            Ok((ship_name, Err(e))) => {
                tracing::error!(ship_name = %ship_name, error = %e, "Ship error occurred");

                if let crate::error::Error::Api(api_error) = &e
                    && api_error.is_universe_reset()
                {
                    self.context.cancellation_tokens.run_cancel_token.cancel();
                } else if let crate::error::Error::ArcError(arc_error) = e
                    && let crate::error::Error::Api(api_error) = &*arc_error
                    && api_error.is_universe_reset()
                {
                    self.context.cancellation_tokens.run_cancel_token.cancel();
                } else {
                    // self.context.cancellation_tokens.global_cancel_token.cancel();
                }
            }
            Err(e) => {
                tracing::error!(error = ?e, "Ship join error");
            }
        }
    }
}

impl Manager for ShipTaskHandler {
    fn run(
        &mut self,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<(), crate::error::Error>> + Send + '_>,
    > {
        Box::pin(async move { self.await_all().await })
    }

    fn get_name(&self) -> &str {
        "ShipTaskHandler"
    }

    fn get_cancel_token(&self) -> &tokio_util::sync::CancellationToken {
        &self.slow_cancel_token
    }
}
