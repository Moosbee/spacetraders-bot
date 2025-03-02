use tokio::{select, task::JoinSet};

use crate::{sql, types::ConductorContext};

use super::Manager;

pub struct ShipTaskHandler {
    receiver: tokio::sync::mpsc::Receiver<sql::ShipInfo>,
    ship_cancel_token: tokio_util::sync::CancellationToken,
    manager_cancel_token: tokio_util::sync::CancellationToken,
    cancel_token: tokio_util::sync::CancellationToken,
    context: ConductorContext,
}

type ShipFuture = ();

#[derive(Debug, Clone)]
pub struct ShipTaskMessanger {
    sender: tokio::sync::mpsc::Sender<sql::ShipInfo>,
}

impl ShipTaskMessanger {
    pub async fn start_ship(&self, ship_names: sql::ShipInfo) {
        log::debug!("start_ship: {:?}", ship_names);
        let _erg = self.sender.send(ship_names).await;
    }
}

impl ShipTaskHandler {
    pub fn create() -> (
        tokio::sync::mpsc::Receiver<sql::ShipInfo>,
        ShipTaskMessanger,
    ) {
        let (sender, receiver) = tokio::sync::mpsc::channel(1024);
        (receiver, ShipTaskMessanger { sender })
    }
    pub fn new(
        ship_cancel_token: tokio_util::sync::CancellationToken,
        manager_cancel_token: tokio_util::sync::CancellationToken,
        cancel_token: tokio_util::sync::CancellationToken,
        context: ConductorContext,
        receiver: tokio::sync::mpsc::Receiver<sql::ShipInfo>,
    ) -> Self {
        log::debug!("ShipTaskHandler::new");
        Self {
            ship_cancel_token,
            receiver,
            manager_cancel_token,
            cancel_token,
            context,
        }
    }

    pub async fn await_all(&mut self) -> Result<(), crate::error::Error> {
        log::debug!("ShipTaskHandler::await_all");
        let mut set: JoinSet<(String, Result<ShipFuture, anyhow::Error>)> = JoinSet::new();

        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        while let Ok(ship_name) = self.receiver.try_recv() {
            log::debug!(
                "ShipTaskHandler::await_all starting: got ship_name: {:?}",
                ship_name
            );
            let pilot = crate::pilot::Pilot::new(
                self.context.clone(),
                ship_name.symbol.clone(),
                self.ship_cancel_token.child_token(),
            );
            set.spawn(async move {
                (
                    ship_name.symbol.clone(),
                    pilot.pilot_ship().await.map_err(anyhow::Error::from),
                )
            });
        }

        loop {
            select! {
                ship_name = self.receiver.recv() => {
                    match ship_name {
                        Some(ship_name) => {
                            log::debug!("ShipTaskHandler::await_all: got ship_name: {:?}", ship_name);
                            let pilot = crate::pilot::Pilot::new(self.context.clone(), ship_name.symbol.clone(), self.ship_cancel_token.child_token());
                             set.spawn(async move {
                                (
                                    ship_name.symbol.clone(),
                                    pilot.pilot_ship().await.map_err(anyhow::Error::from),
                                )
                            });
                        }
                        None => {
                            log::debug!("ShipTaskHandler::await_all: receiver is closed");
                            break;
                        }
                    }
                }
                finished_future = set.join_next() => {
                  match finished_future {
                    Some(finished_future) => {
                      match finished_future {
                        Ok((ship_name,Ok(erg))) => {
                          log::debug!("ShipTaskHandler::await_all: finished_future: {:?} last one: {}", erg,ship_name);
                        }
                        Ok((ship_name,Err(e))) => {
                          log::error!(
                              "Ship error for ship: {} error: {} {:?} {:?} {:?}",
                              ship_name,
                              e,
                              e.backtrace(),
                              e.source(),
                              e.root_cause()
                          );
                        }
                        Err(e) => {
                          log::error!("JoinError Error: {}", e);
                        }
                      }
                    },
                    None => {
                        log::debug!("ShipTaskHandler::await_all: finished_future is None");
                        break;
                    }
                  }
                }
            }
        }

        log::debug!("ShipTaskHandler::await_all finished");
        self.manager_cancel_token.cancel();
        Ok(())
    }
}

impl Manager for ShipTaskHandler {
    fn run(
        &mut self,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<(), crate::error::Error>> + Send + '_>,
    > {
        log::debug!("ShipTaskHandler::run");
        Box::pin(async move { self.await_all().await })
    }

    fn get_name(&self) -> &str {
        "ShipTaskHandler"
    }

    fn get_cancel_token(&self) -> &tokio_util::sync::CancellationToken {
        &self.cancel_token
    }
}
