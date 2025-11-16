use std::sync::{atomic::AtomicI32, Arc};

use database::DatabaseConnector;
use futures::FutureExt;
use rand::seq::SliceRandom;
use ship::status::{ExtractorState, MiningShipAssignment};
use space_traders_client::models;
use tracing::debug;
use tracing::instrument;

use crate::{
    error::Result,
    manager::mining_manager::{ActionType, ExtractorTransferRequest, TransferResult},
    utils::ConductorContext,
};

pub struct ExtractionPilot {
    count: Arc<AtomicI32>,
    context: ConductorContext,
}

impl ExtractionPilot {
    pub fn new(context: ConductorContext) -> Self {
        Self {
            count: Arc::new(AtomicI32::new(0)),
            context,
        }
    }

    async fn update_assignment(
        &self,
        ship: &mut ship::MyShip,
        is_syphon: bool,
        state: ExtractorState,
        waypoint_symbol: Option<String>,
        extractions: Option<i32>,
        notify: bool,
    ) {
        let assignment = if is_syphon {
            MiningShipAssignment::Siphoner {
                state,
                waypoint_symbol,
                extractions,
            }
        } else {
            MiningShipAssignment::Extractor {
                state,
                waypoint_symbol,
                extractions,
            }
        };

        ship.status.status = ship::AssignmentStatus::Mining { assignment };
        if notify {
            ship.notify(true).await;
        }
    }

    #[instrument(level = "info", name = "spacetraders::pilot::mining::pilot_extraction", skip(self, pilot, ship), fields(self.ship_symbol = pilot.ship_symbol, waypoint))]
    pub async fn execute_extraction_circle(
        &self,
        ship: &mut ship::MyShip,
        pilot: &crate::pilot::Pilot,
        is_syphon: bool,
    ) -> Result<()> {
        debug!("Executing extraction circle for ship: {}", ship.symbol);
        let waypoint_symbol = self
            .context
            .mining_manager
            .get_waypoint(ship, is_syphon)
            .await?;

        tracing::Span::current().record("waypoint", &waypoint_symbol);

        debug!("Mining Waypoint: {}", waypoint_symbol);

        self.update_assignment(
            ship,
            is_syphon,
            ExtractorState::Unknown,
            Some(waypoint_symbol.clone()),
            Some(self.count.load(std::sync::atomic::Ordering::Relaxed)),
            true,
        )
        .await;

        self.go_to_waypoint(ship, &waypoint_symbol, is_syphon)
            .await?;

        self.update_assignment(
            ship,
            is_syphon,
            ExtractorState::OnCooldown,
            Some(waypoint_symbol.clone()),
            Some(self.count.load(std::sync::atomic::Ordering::Relaxed)),
            false,
        )
        .await;

        self.context
            .mining_manager
            .notify_waypoint(ship, is_syphon)
            .await?;
        let mut rec = self
            .context
            .mining_manager
            .extractor_contact(&ship.symbol)
            .await?;

        let i = self.wait_for_extraction(ship, pilot, &mut rec).await?;

        if i == 0 {
            debug!("Extraction Cancelled for ship: {}", ship.symbol);
            self.context.mining_manager.unassign_waypoint(ship).await?;
            return Ok(());
        }

        let done = if !self.has_space(ship) {
            debug!("No space on ship: {}", ship.symbol);

            self.update_assignment(
                ship,
                is_syphon,
                ExtractorState::InvFull,
                Some(waypoint_symbol.clone()),
                Some(self.count.load(std::sync::atomic::Ordering::Relaxed)),
                true,
            )
            .await;

            let pin_sleep = tokio::time::sleep(std::time::Duration::from_millis(
                5000 + rand::random::<u64>() % 10000,
            ));
            let pin_sleep_pined = std::pin::pin!(pin_sleep);

            let i = self
                .wait_for(ship, pilot, &mut rec, pin_sleep_pined)
                .await?;
            if i == 0 {
                debug!("No extraction after waiting for ship: {}", ship.symbol);
                self.context.mining_manager.unassign_waypoint(ship).await?;
                return Ok(());
            }
            0
        } else {
            self.update_assignment(
                ship,
                is_syphon,
                ExtractorState::Mining,
                Some(waypoint_symbol.clone()),
                Some(self.count.load(std::sync::atomic::Ordering::Relaxed)),
                false,
            )
            .await;
            self.extract(ship, is_syphon).await?;
            self.eject_blacklist(ship).await?;

            1
        };

        self.update_assignment(
            ship,
            is_syphon,
            ExtractorState::OnCooldown,
            Some(waypoint_symbol.clone()),
            Some(self.count.load(std::sync::atomic::Ordering::Relaxed)),
            true,
        )
        .await;

        if done == 1 || rand::random::<u64>() % 10 == 0 {
            self.context
                .mining_manager
                .extraction_complete(&ship.symbol, &ship.nav.waypoint_symbol)
                .await?;
        }

        let _i = self.wait_for_extraction(ship, pilot, &mut rec).await?;

        debug!("Dropping extractor contact for ship: {}", ship.symbol);

        drop(rec);

        self.context.mining_manager.unassign_waypoint(ship).await?;

        self.update_assignment(
            ship,
            is_syphon,
            ExtractorState::Unknown,
            Some(waypoint_symbol.clone()),
            Some(self.count.load(std::sync::atomic::Ordering::Relaxed)),
            false,
        )
        .await;

        debug!("Extraction circle complete for ship: {}", ship.symbol);
        Ok(())
    }

    async fn go_to_waypoint(
        &self,
        ship: &mut ship::MyShip,
        waypoint_symbol: &str,
        is_syphon: bool,
    ) -> Result<()> {
        debug!("Going to waypoint: {}", waypoint_symbol);

        if ship.nav.waypoint_symbol == waypoint_symbol && !ship.nav.is_in_transit() {
            debug!("Already at waypoint: {}", waypoint_symbol);
            return Ok(());
        }

        self.update_assignment(
            ship,
            is_syphon,
            ExtractorState::InTransit,
            Some(waypoint_symbol.to_string()),
            None,
            true,
        )
        .await;

        ship.wait_for_arrival_mut(&self.context.api)
            .await
            .map_err(|e| e.to_string())?;
        if ship.nav.waypoint_symbol == waypoint_symbol {
            debug!("Already at waypoint: {}", waypoint_symbol);
            return Ok(());
        }

        let system_symbol = ship.nav.system_symbol.clone();

        if !waypoint_symbol.contains(&system_symbol) {
            return Err("Waypoint is not in ship's system".into());
        }

        let budget_manager = self.context.budget_manager.clone();

        let update_funds_fn = move |amount| budget_manager.set_current_funds(amount);

        ship.nav_to(
            waypoint_symbol,
            true,
            database::TransactionReason::MiningWaypoint(waypoint_symbol.to_string()),
            &self.context.database_pool,
            &self.context.api,
            update_funds_fn,
        )
        .await?;

        debug!("Arrived at waypoint: {}", waypoint_symbol);
        Ok(())
    }

    fn has_space(&self, ship: &mut ship::MyShip) -> bool {
        debug!(ship_symbol = %ship.symbol, capacity = ship.cargo.capacity, units = ship.cargo.units, "Checking space on ship");
        ship.cargo.units < ship.cargo.capacity
    }

    async fn extract(&self, ship: &mut ship::MyShip, is_syphon: bool) -> Result<()> {
        debug!("Extracting on ship: {}", ship.symbol);
        if ship.is_on_cooldown() {
            debug!("Ship is on cooldown: {}", ship.symbol);
            return Err("Ship is on cooldown".into());
        }

        ship.ensure_undocked(&self.context.api).await?;

        let action = if is_syphon {
            ActionType::Siphon
        } else {
            ActionType::Extract
        };
        // let action = ActionType::get_action(&ship).ok_or("Invalid ship role")?;

        let state_id = ship.snapshot(&self.context.database_pool).await?;

        match action {
            ActionType::Extract => {
                let survey: Option<database::Survey> = self.get_best_survey(ship).await?;

                if let Some(survey) = survey {
                    let prefer_list =
                        { self.context.config.read().await.mining_prefer_list.clone() };

                    let count_a = survey.get_percent();
                    let score_a = count_a
                        .iter()
                        .filter(|f| prefer_list.contains(&f.0))
                        .map(|f| f.1)
                        .sum::<f64>();
                    debug!(survey_signature = %survey.signature, ship_symbol = %ship.symbol, score = score_a, "Using survey on ship");

                    let survey_erg = ship
                        .extract_with_survey(&self.context.api, &(&survey).into())
                        .await;

                    match survey_erg {
                        Err(space_traders_client::apis::Error::ResponseError(e)) => {
                            let error_code = e.get_error_code();
                            if error_code
                                .map(|code| {
                                    code == models::error_codes::SHIP_EXTRACT_DESTABILIZED_ERROR
                                })
                                .unwrap_or(false)
                            {
                                tracing::warn!(
                                    ship_symbol = ship.symbol,
                                    waypoint_symbol = ship.nav.waypoint_symbol,
                                    "Waypoint destabilized",
                                );

                                let new_wp = database::Waypoint::get_by_symbol(
                                    &self.context.database_pool,
                                    &ship.nav.waypoint_symbol,
                                )
                                .await?;
                                let mut wp = if let Some(new_wp) = new_wp {
                                    new_wp
                                } else {
                                    let new_wp = self
                                        .context
                                        .api
                                        .get_waypoint(
                                            &ship.nav.system_symbol,
                                            &ship.nav.waypoint_symbol,
                                        )
                                        .await?;
                                    (&(*new_wp.data)).into()
                                };
                                wp.unstable_since = Some(chrono::Utc::now());
                                database::Waypoint::insert(&self.context.database_pool, &wp)
                                    .await?;
                            } else if error_code
                                .map(|code| {
                                    code == models::error_codes::SHIP_SURVEY_EXHAUSTED_ERROR
                                })
                                .unwrap_or(false)
                            {
                                let mut survey = survey.clone();
                                tracing::warn!(
                                    signature = survey.signature,
                                    ship_symbol = ship.symbol,
                                    "Survey exhausted",
                                );
                                survey.exhausted_since = Some(chrono::Utc::now());
                                database::Survey::insert(&self.context.database_pool, &survey)
                                    .await?;
                            } else if error_code
                                .map(|code| {
                                    code == models::error_codes::SHIP_SURVEY_EXPIRATION_ERROR
                                })
                                .unwrap_or(false)
                            {
                                // no real action needed
                                debug!("Survey {} has expired", survey.signature);
                            } else {
                                return Err(
                                    space_traders_client::apis::Error::ResponseError(e).into()
                                );
                            }
                        }
                        Err(e) => return Err(e.into()),
                        Ok(erg) => {
                            let now = chrono::Utc::now();
                            if true {
                                ship.reload(&self.context.api).await?;
                            }
                            let after_state_id = ship.snapshot(&self.context.database_pool).await?;

                            let extraction = database::Extraction {
                                id: 0,
                                ship_symbol: ship.symbol.clone(),
                                waypoint_symbol: ship.nav.waypoint_symbol.clone(),
                                ship_info_before: state_id,
                                ship_info_after: after_state_id,
                                siphon: false,
                                yield_symbol: erg.data.extraction.r#yield.symbol,
                                yield_units: erg.data.extraction.r#yield.units,
                                survey: Some(survey.signature.clone()),
                                created_at: now,
                            };

                            database::Extraction::insert(&self.context.database_pool, &extraction)
                                .await?;

                            tracing::info!(
                                "Extracted on ship: {} erg {:?} events: {:?}",
                                erg.data.extraction.ship_symbol,
                                erg.data.extraction.r#yield,
                                erg.data.events
                            );
                        }
                    }
                } else {
                    let simple_erg = ship.extract(&self.context.api).await;

                    match simple_erg {
                        Err(space_traders_client::apis::Error::ResponseError(e)) => {
                            let error_code = e.get_error_code();
                            if error_code
                                .map(|e| e == models::error_codes::SHIP_EXTRACT_DESTABILIZED_ERROR)
                                .unwrap_or(false)
                            {
                                tracing::warn!(
                                    ship_symbol = ship.symbol,
                                    waypoint_symbol = ship.nav.waypoint_symbol,
                                    "Waypoint destabilized",
                                );

                                let new_wp = database::Waypoint::get_by_symbol(
                                    &self.context.database_pool,
                                    &ship.nav.waypoint_symbol,
                                )
                                .await?;
                                let mut wp = if let Some(new_wp) = new_wp {
                                    new_wp
                                } else {
                                    let new_wp = self
                                        .context
                                        .api
                                        .get_waypoint(
                                            &ship.nav.system_symbol,
                                            &ship.nav.waypoint_symbol,
                                        )
                                        .await?;
                                    (&(*new_wp.data)).into()
                                };
                                wp.unstable_since = Some(chrono::Utc::now());
                                database::Waypoint::insert(&self.context.database_pool, &wp)
                                    .await?;
                            } else {
                                return Err(
                                    space_traders_client::apis::Error::ResponseError(e).into()
                                );
                            }
                        }
                        Err(e) => return Err(e.into()),
                        Ok(erg) => {
                            let now = chrono::Utc::now();
                            if true {
                                ship.reload(&self.context.api).await?;
                            }
                            let after_state_id = ship.snapshot(&self.context.database_pool).await?;

                            let extraction = database::Extraction {
                                id: 0,
                                ship_symbol: ship.symbol.clone(),
                                waypoint_symbol: ship.nav.waypoint_symbol.clone(),
                                ship_info_before: state_id,
                                ship_info_after: after_state_id,
                                siphon: false,
                                yield_symbol: erg.data.extraction.r#yield.symbol,
                                yield_units: erg.data.extraction.r#yield.units,
                                survey: None,
                                created_at: now,
                            };

                            database::Extraction::insert(&self.context.database_pool, &extraction)
                                .await?;

                            tracing::info!(
                                "Extracted on ship: {} erg {:?} events: {:?}",
                                erg.data.extraction.ship_symbol,
                                erg.data.extraction.r#yield,
                                erg.data.events
                            );
                        }
                    }
                };
            }
            ActionType::Siphon => {
                let now = chrono::Utc::now();
                if true {
                    ship.reload(&self.context.api).await?;
                }
                let erg = ship.siphon(&self.context.api).await?;

                let after_state_id = ship.snapshot(&self.context.database_pool).await?;

                let extraction = database::Extraction {
                    id: 0,
                    ship_symbol: ship.symbol.clone(),
                    waypoint_symbol: ship.nav.waypoint_symbol.clone(),
                    ship_info_before: state_id,
                    ship_info_after: after_state_id,
                    siphon: true,
                    survey: None,

                    yield_symbol: erg.data.siphon.r#yield.symbol,
                    yield_units: erg.data.siphon.r#yield.units,
                    created_at: now,
                };

                database::Extraction::insert(&self.context.database_pool, &extraction).await?;

                tracing::info!(
                    "Siphoned on ship: {} erg {:?} events: {:?}",
                    erg.data.siphon.ship_symbol,
                    erg.data.siphon.r#yield,
                    erg.data.events
                );
            }
        }

        self.count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        debug!(
            "Count after extraction: {}",
            self.count.load(std::sync::atomic::Ordering::Relaxed)
        );

        ship.notify(true).await;

        debug!("Extracted on ship: {}", ship.symbol);
        Ok(())
    }

    async fn eject_blacklist(&self, ship: &mut ship::MyShip) -> Result<()> {
        debug!("Ejecting blacklist on ship: {}", ship.symbol);
        let cargo = ship.cargo.inventory.clone();
        let eject_list = &self.context.config.read().await.mining_eject_list;
        for item in cargo.iter() {
            if eject_list.contains(item.0) {
                debug!("Ejecting: {:?}", item);
                ship.jettison(&self.context.api, *item.0, *item.1).await?;
            }
        }
        Ok(())
    }

    async fn wait_for_extraction(
        &self,
        ship: &mut ship::MyShip,
        pilot: &crate::pilot::Pilot,
        receiver: &mut tokio::sync::mpsc::Receiver<ExtractorTransferRequest>,
    ) -> Result<i32> {
        let ship_future = { ship.wait_for_cooldown() };
        let ship_future_pined = std::pin::pin!(ship_future);
        let erg = self
            .wait_for(ship, pilot, receiver, ship_future_pined)
            .await?;
        Ok(erg)
    }

    async fn wait_for(
        &self,
        ship: &mut ship::MyShip,
        pilot: &crate::pilot::Pilot,
        receiver: &mut tokio::sync::mpsc::Receiver<ExtractorTransferRequest>,
        future: impl std::future::Future<Output = ()> + Unpin,
    ) -> Result<i32> {
        //needs revisit

        // tell mining manager you can transfer your cargo
        // wait until cooldown is done
        //    in meantime, listen to mining manager and transfer cargo it tells you
        // cut connection to mining manager

        let mut fused_future = future.fuse();

        loop {
            let i = {
                tokio::select! {
                    _ = pilot.cancellation_token.cancelled() => {(None,0)},// it's the end of the Programm we don't care(for now)
                    _ = &mut fused_future => {(None,1)},
                    msg = receiver.recv() => {(msg,2)},
                }
            };

            match i {
                (Some(msg), _) => {
                    self.handle_extractor_transfer_request(ship, msg).await?;
                }
                (None, 1) => {
                    debug!("Cooldown is done for ship: {}", ship.symbol);
                    return Ok(1);
                }
                (None, 2) => {
                    debug!("No more messages for ship: {}", ship.symbol);
                    return Ok(2);
                }
                (None, _) => {
                    debug!("No more messages for ship: {}", ship.symbol);
                    return Ok(0);
                }
            }
        }
    }

    async fn handle_extractor_transfer_request(
        &self,
        ship: &mut ship::MyShip,
        request: ExtractorTransferRequest,
    ) -> Result<()> {
        debug!("Handling transfer request for ship: {}", ship.symbol);

        if ship.cargo.get_amount(&request.trade_symbol) < request.amount {
            tracing::warn!(
                ship_symbol = ship.symbol,
                to_symbol = request.to_symbol,
                trade_symbol = request.trade_symbol.to_string(),
                amount = request.amount,
                "Not enough cargo to transfer",
            );
            let _erg = request.callback.send(None);
            return Ok(());
        }

        let erg = ship
            .simple_transfer_cargo(
                request.trade_symbol,
                request.amount,
                &self.context.api,
                &request.to_symbol,
            )
            .await;

        let transfer_result = match erg {
            Ok(_erg) => Some(TransferResult {
                trade_symbol: request.trade_symbol,
                units: request.amount,
            }),
            Err(error) => {
                tracing::error!(
                    ship_symbol = ship.symbol,
                    to_symbol = request.to_symbol,
                    error= format!("{} {:?}", error, error),
                    request=?request,
                    "Transfer request failed",
                );

                None
            }
        };

        let _erg = request.callback.send(transfer_result);

        debug!(ship_symbol = %ship.symbol, to_symbol = %request.to_symbol, "Handled transfer request for ship");
        Ok(())
    }

    async fn get_best_survey(&self, ship: &mut ship::MyShip) -> Result<Option<database::Survey>> {
        let mut working_surveys = database::Survey::get_working_for_waypoint(
            &self.context.database_pool,
            &ship.nav.waypoint_symbol,
        )
        .await?;

        working_surveys.shuffle(&mut rand::thread_rng());

        let prefer_list = { self.context.config.read().await.mining_prefer_list.clone() };

        let best_survey = working_surveys.iter().max_by(|a, b| {
            let count_a = a.get_percent();
            let score_a = count_a
                .iter()
                .filter(|f| prefer_list.contains(&f.0))
                .map(|f| f.1)
                .sum::<f64>();
            let count_b = b.get_percent();
            let score_b = count_b
                .iter()
                .filter(|f| prefer_list.contains(&f.0))
                .map(|f| f.1)
                .sum::<f64>();

            score_b.partial_cmp(&score_a).unwrap()
        });

        best_survey.cloned().map_or(Ok(None), |f| Ok(Some(f)))
    }
}
