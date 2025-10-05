use std::{
    collections::HashMap,
    sync::atomic::{AtomicI64, Ordering},
};

use database::{DatabaseConnector, FundStatus, ReservedFund};
use tokio::sync::Mutex;

#[derive(Debug, Clone, serde::Serialize)]
pub struct BudgetInfo {
    pub current_funds: i64,
    pub iron_reserve: i64,
    pub reserved_amount: i64,
    pub spendable: i64,
    pub reservations: Vec<ReservedFund>,
}

#[derive(Debug)]
pub struct BudgetManager {
    current_funds: AtomicI64,
    reserved_funds: Mutex<HashMap<i64, ReservedFund>>,
    iron_reserve: i64,
    // update_funds_fn: Option<Fn(i64) + Send + Sync>,
}

impl BudgetManager {
    pub async fn init(
        database_pool: &database::DbPool,
        current_funds: i64,
        iron_reserve: i64,
    ) -> crate::error::Result<Self> {
        let reserved_funds =
            ReservedFund::get_by_status(database_pool, FundStatus::Reserved).await?;

        Ok(BudgetManager {
            current_funds: AtomicI64::new(current_funds),
            reserved_funds: Mutex::new(reserved_funds.into_iter().map(|rf| (rf.id, rf)).collect()),
            iron_reserve,
        })
    }

    pub async fn get_budget_info(&self) -> BudgetInfo {
        let reserved_funds = self.reserved_funds.lock().await;
        let reserved_amount = Self::get_still_reserved_funds(reserved_funds.clone());
        let spendable =
            self.current_funds.load(Ordering::SeqCst) - self.iron_reserve - reserved_amount;

        BudgetInfo {
            current_funds: self.current_funds.load(Ordering::SeqCst),
            iron_reserve: self.iron_reserve,
            reserved_amount,
            spendable,
            reservations: reserved_funds.values().cloned().collect(),
        }
    }

    fn get_still_reserved_funds(reserved_funds: HashMap<i64, ReservedFund>) -> i64 {
        reserved_funds
            .into_values()
            .filter(|rf| rf.status == FundStatus::Reserved)
            .map(|rf| (rf.amount - rf.actual_amount).max(0)) // the reserved amount - the already used amount
            .sum()
    }

    pub fn get_current_funds(&self) -> i64 {
        self.current_funds.load(Ordering::SeqCst)
    }

    pub fn set_current_funds(&self, amount: i64) {
        self.current_funds.store(amount, Ordering::SeqCst);
    }

    pub async fn can_reserve_funds(&self, amount: i64) -> bool {
        let spendable = self.get_spendable_funds().await;
        spendable >= amount
    }

    pub async fn get_spendable_funds(&self) -> i64 {
        let reserved_funds = self.reserved_funds.lock().await;
        let reserved_amount = Self::get_still_reserved_funds(reserved_funds.clone());
        self.current_funds.load(Ordering::SeqCst) - self.iron_reserve - reserved_amount
    }

    pub async fn reserve_funds(
        &self,
        database_pool: &database::DbPool,
        amount: i64,
    ) -> Result<ReservedFund, crate::error::Error> {
        self.reserve_funds_with_remain(database_pool, amount, self.iron_reserve)
            .await
    }

    pub async fn reserve_funds_with_remain(
        &self,
        database_pool: &database::DbPool,
        amount: i64,
        remain: i64,
    ) -> Result<ReservedFund, crate::error::Error> {
        tracing::debug!(
            "Attempting to reserve funds: {}, with remain: {}",
            amount,
            remain
        );
        let mut reserved_funds = self.reserved_funds.lock().await;
        let reserved_amount = Self::get_still_reserved_funds(reserved_funds.clone());
        let spendable = self.current_funds.load(Ordering::SeqCst) - remain - reserved_amount;
        if spendable < amount {
            return Err(crate::error::Error::NotEnoughFunds {
                remaining_funds: spendable,
                required_funds: amount,
            });
        }

        let mut funds = ReservedFund {
            id: 0,
            amount,
            status: FundStatus::Reserved,
            actual_amount: 0,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let reserved_fund_id = ReservedFund::insert_new(database_pool, funds.clone()).await?;

        funds.id = reserved_fund_id;

        reserved_funds.insert(funds.id, funds.clone());
        Ok(funds)
    }

    pub async fn cancel_reservation(
        &self,
        database_pool: &database::DbPool,
        reservation_id: i64,
    ) -> Result<(), crate::error::Error> {
        let mut reserved_funds = self.reserved_funds.lock().await;
        tracing::debug!("Cancelling reservation id: {}", reservation_id);

        {
            let reserved_fund = reserved_funds
                .get_mut(&reservation_id)
                .ok_or_else(|| crate::error::Error::ReservationNotFound { reservation_id })?;

            let db_funds = ReservedFund::get_by_id(database_pool, &reservation_id)
                .await?
                .ok_or_else(|| crate::error::Error::ReservationNotFound { reservation_id })?;

            if reserved_fund != &db_funds {
                return Err("Reservation mismatch".to_string().into());
            }

            reserved_fund.status = FundStatus::Cancelled;

            reserved_fund.updated_at = chrono::Utc::now();

            ReservedFund::insert(database_pool, reserved_fund).await?;
        }

        reserved_funds.remove(&reservation_id);

        Ok(())
    }

    pub async fn use_reservation(
        &self,
        database_pool: &database::DbPool,
        reservation_id: i64,
        increment_amount: i64,
    ) -> Result<(), crate::error::Error> {
        let mut reserved_funds = self.reserved_funds.lock().await;
        tracing::debug!("Using reservation id: {}", reservation_id);

        let reserved_fund = reserved_funds
            .get_mut(&reservation_id)
            .ok_or_else(|| crate::error::Error::ReservationNotFound { reservation_id })?;

        let db_funds = ReservedFund::get_by_id(database_pool, &reservation_id)
            .await?
            .ok_or_else(|| crate::error::Error::ReservationNotFound { reservation_id })?;

        if reserved_fund != &db_funds {
            return Err("Reservation mismatch".to_string().into());
        }

        if reserved_fund.status != FundStatus::Reserved {
            return Err("Reservation is not in a usable state".to_string().into());
        }

        reserved_fund.actual_amount += increment_amount;

        reserved_fund.updated_at = chrono::Utc::now();

        ReservedFund::insert(database_pool, reserved_fund).await?;

        // reserved_funds.remove(&reservation_id);

        Ok(())
    }

    pub async fn complete_use_reservation(
        &self,
        database_pool: &database::DbPool,
        reservation_id: i64,
        actual_amount: i64,
    ) -> Result<(), crate::error::Error> {
        let mut reserved_funds = self.reserved_funds.lock().await;
        tracing::debug!("Completing use of reservation id: {}", reservation_id);

        let reserved_fund = reserved_funds
            .get_mut(&reservation_id)
            .ok_or_else(|| crate::error::Error::ReservationNotFound { reservation_id })?;

        let db_funds = ReservedFund::get_by_id(database_pool, &reservation_id)
            .await?
            .ok_or_else(|| crate::error::Error::ReservationNotFound { reservation_id })?;

        if reserved_fund != &db_funds {
            return Err("Reservation mismatch".to_string().into());
        }

        if reserved_fund.status != FundStatus::Reserved {
            return Err("Reservation is not in a usable state".to_string().into());
        }

        reserved_fund.actual_amount = actual_amount;

        reserved_fund.status = FundStatus::Used;

        reserved_fund.updated_at = chrono::Utc::now();

        ReservedFund::insert(database_pool, reserved_fund).await?;

        reserved_funds.remove(&reservation_id);

        Ok(())
    }

    pub async fn complete_reservation(
        &self,
        database_pool: &database::DbPool,
        reservation_id: i64,
    ) -> Result<(), crate::error::Error> {
        let mut reserved_funds = self.reserved_funds.lock().await;
        tracing::debug!("Completing reservation id: {}", reservation_id);

        let reserved_fund = reserved_funds
            .get_mut(&reservation_id)
            .ok_or_else(|| crate::error::Error::ReservationNotFound { reservation_id })?;

        let db_funds = ReservedFund::get_by_id(database_pool, &reservation_id)
            .await?
            .ok_or_else(|| crate::error::Error::ReservationNotFound { reservation_id })?;

        if reserved_fund != &db_funds {
            return Err("Reservation mismatch".to_string().into());
        }

        if reserved_fund.status != FundStatus::Reserved {
            return Err("Reservation is not in a usable state".to_string().into());
        }

        reserved_fund.status = FundStatus::Used;

        reserved_fund.updated_at = chrono::Utc::now();

        ReservedFund::insert(database_pool, reserved_fund).await?;

        reserved_funds.remove(&reservation_id);

        Ok(())
    }
}
