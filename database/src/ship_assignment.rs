use tracing::instrument;

use crate::{DatabaseConnector, DbPool};

#[derive(Debug, Clone, PartialEq, Eq, async_graphql::SimpleObject)]
#[graphql(name = "DBShipAssignment")]
pub struct ShipAssignment {
    pub id: i64,
    pub fleet_id: i32,
    pub priority: i32, // lower numbers are higher priority, distributed around 100 +- 50
    pub max_purchase_price: i32, // between 900_000 and 20_000_000
    pub credits_threshold: i32, // between 50_000 and 5_000_000
    pub disabled: bool, // if true, ship should not be assigned to this assignment and currently assigned ships should be removed
    pub range_min: i32, // aka fuel capacity minimum, -1 means infinite, between -1 and 5000
    pub cargo_min: i32, // minimum cargo space required, 0 means no requirement, between 0 and 1000
    pub survey: bool,
    pub extractor: bool,
    pub siphon: bool,
    pub warp_drive: bool,
    // pub refinery: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SimpleShipRequirement {
    pub max_purchase_price: i32,
    pub credits_threshold: i32,
    pub disabled: bool,
    pub range_min: i32,
    pub cargo_min: i32,
    pub survey: bool,
    pub extractor: bool,
    pub siphon: bool,
    pub warp_drive: bool,
    // pub refinery: bool,
}

impl From<ShipAssignment> for SimpleShipRequirement {
    fn from(value: ShipAssignment) -> Self {
        Self {
            max_purchase_price: value.max_purchase_price,
            credits_threshold: value.credits_threshold,
            disabled: value.disabled,
            range_min: value.range_min,
            cargo_min: value.cargo_min,
            survey: value.survey,
            extractor: value.extractor,
            siphon: value.siphon,
            warp_drive: value.warp_drive,
        }
    }
}

impl ShipAssignment {
    /// assignments can merge, ie updated without invalidation assigned ships when:
    /// - they belong to the same fleet
    /// - survey is the same
    /// - extractor is the same
    /// - siphon is the same
    /// - warp_drive is the same
    /// # - refinery is the same
    /// - range may change, except from infinite to finite or vice versa
    /// - cargo_min may change, but not from 0 to non-0 or vice versa
    /// - max_purchase_price may change
    /// - credits_threshold may change
    /// - priority may change
    /// - disabled may change
    pub fn can_merge(&self, b: &ShipAssignment) -> bool {
        if self.fleet_id != b.fleet_id {
            return false;
        }
        if self.survey != b.survey {
            return false;
        }
        if self.extractor != b.extractor {
            return false;
        }
        if self.siphon != b.siphon {
            return false;
        }
        if self.warp_drive != b.warp_drive {
            return false;
        }
        // if self.refinery != b.refinery {
        //     return false;
        // }
        if (self.range_min < 0 && b.range_min >= 0) || (self.range_min >= 0 && b.range_min < 0) {
            return false;
        }
        if (self.cargo_min == 0 && b.cargo_min > 0) || (self.cargo_min > 0 && b.cargo_min == 0) {
            return false;
        }
        true
    }

    /// calculates a score for merging two assignments, lower is better
    /// if they perfectly match, the bes solution the score is 0.0
    /// otherwise the score increases with the difference in modifiable fields
    /// fields that cannot be merged return f32::INFINITY
    /// priority difference is weighted highest
    /// then range_min
    /// then cargo_min
    /// then max_purchase_price
    /// then credits_threshold
    pub fn merge_score(&self, b: &ShipAssignment) -> f32 {
        const PRIORITY_MULTIPLIER: f32 = 0.04; // range ~100, highest priority
        const RANGE_MULTIPLIER: f32 = 0.008; // range ~5000, second highest
        const CARGO_MULTIPLIER: f32 = 0.004; // range ~1000, third
        const MAX_PRICE_MULTIPLIER: f32 = 0.0000005; // range ~19_100_000, fourth
        const CREDITS_MULTIPLIER: f32 = 0.0000002; // range ~4_950_000, lowest priority
        let priority_diff = (self.priority - b.priority).abs() as f32 * PRIORITY_MULTIPLIER;
        let range_diff = (self.range_min - b.range_min).abs() as f32 * RANGE_MULTIPLIER;
        let cargo_diff = (self.cargo_min - b.cargo_min).abs() as f32 * CARGO_MULTIPLIER;
        let max_price_diff =
            (self.max_purchase_price - b.max_purchase_price).abs() as f32 * MAX_PRICE_MULTIPLIER;
        let credits_diff =
            (self.credits_threshold - b.credits_threshold).abs() as f32 * CREDITS_MULTIPLIER;
        priority_diff + range_diff + cargo_diff + max_price_diff + credits_diff
    }

    /// merges the modifiable fields from other into self
    pub fn merge_into(&mut self, other: &ShipAssignment) {
        self.priority = other.priority;
        self.max_purchase_price = other.max_purchase_price;
        self.credits_threshold = other.credits_threshold;
        // disabled is disabled, if any of the two is disabled
        self.disabled = self.disabled || other.disabled;
        self.range_min = other.range_min;
        self.cargo_min = other.cargo_min;
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    pub async fn get_by_id(
        database_pool: &DbPool,
        id: i64,
    ) -> crate::Result<Option<ShipAssignment>> {
        let resp = sqlx::query_as!(
            ShipAssignment,
            r#"
                SELECT
                  id,
                  fleet_id,
                  priority,
                  max_purchase_price,
                  credits_threshold,
                  disabled,
                  range_min,
                  cargo_min,
                  survey,
                  extractor,
                  siphon,
                  warp_drive
                FROM ship_assignment
                WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&database_pool.database_pool)
        .await?;

        Ok(resp)
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    pub async fn get_by_fleet_id(
        database_pool: &DbPool,
        fleet_id: i32,
    ) -> crate::Result<Vec<ShipAssignment>> {
        let resp = sqlx::query_as!(
            ShipAssignment,
            r#"
                SELECT
                  id,
                  fleet_id,
                  priority,
                  max_purchase_price,
                  credits_threshold,
                  disabled,
                  range_min,
                  cargo_min,
                  survey,
                  extractor,
                  siphon,
                  warp_drive
                FROM ship_assignment
                WHERE fleet_id = $1
            "#,
            fleet_id
        )
        .fetch_all(&database_pool.database_pool)
        .await?;

        Ok(resp)
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    pub async fn get_open_assignments(
        database_pool: &DbPool,
    ) -> crate::Result<Vec<ShipAssignment>> {
        // get all "open" assignments from the database, i.e. assignments that are not yet assigned to a ship, that are not disabled and where the fleet is activated
        let resp = sqlx::query_as!(
            ShipAssignment,
            r#"
                SELECT
                  sa.id,
                  sa.fleet_id,
                  sa.priority,
                  max_purchase_price,
                  credits_threshold,
                  sa.disabled,
                  sa.range_min,
                  sa.cargo_min,
                  sa.survey,
                  sa.extractor,
                  sa.siphon,
                  sa.warp_drive
                FROM ship_assignment sa
                JOIN fleet f ON sa.fleet_id = f.id
                left JOIN ship_info si ON (sa.id = si.assignment_id OR sa.id = si.temp_assignment_id)
                WHERE sa.disabled = false AND f.active = true AND si.symbol IS NULL
            "#,
        )
        .fetch_all(&database_pool.database_pool)
        .await?;

        Ok(resp)
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    pub async fn insert_new(database_pool: &DbPool, item: &ShipAssignment) -> crate::Result<i64> {
        let erg = sqlx::query!(
            r#"
                INSERT INTO ship_assignment (
                  fleet_id,
                  priority,
                  max_purchase_price,
                  credits_threshold,
                  disabled,
                  range_min,
                  cargo_min,
                  survey,
                  extractor,
                  siphon,
                  warp_drive
                )
                VALUES (
                  $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11
                )
                RETURNING id
            "#,
            &item.fleet_id,
            &item.priority,
            &item.max_purchase_price,
            &item.credits_threshold,
            &item.disabled,
            &item.range_min,
            &item.cargo_min,
            &item.survey,
            &item.extractor,
            &item.siphon,
            &item.warp_drive,
        )
        .fetch_one(&database_pool.database_pool)
        .await?;

        Ok(erg.id)
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    pub async fn delete_by_id(database_pool: &DbPool, id: i64) -> crate::Result<()> {
        sqlx::query!(
            r#"
                DELETE FROM ship_assignment
                WHERE id = $1
            "#,
            id
        )
        .execute(&database_pool.database_pool)
        .await?;
        Ok(())
    }
}

impl DatabaseConnector<ShipAssignment> for ShipAssignment {
    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn insert(database_pool: &DbPool, item: &ShipAssignment) -> crate::Result<()> {
        sqlx::query!(
            r#"
                INSERT INTO ship_assignment (
                  id,
                  fleet_id,
                  priority,
                  max_purchase_price,
                  credits_threshold,
                  disabled,
                  range_min,
                  cargo_min,
                  survey,
                  extractor,
                  siphon,
                  warp_drive
                )
                VALUES (
                  $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12
                )
                ON CONFLICT (id) DO UPDATE SET
                  fleet_id = EXCLUDED.fleet_id,
                  disabled = EXCLUDED.disabled,
                  priority = EXCLUDED.priority,
                  max_purchase_price = EXCLUDED.max_purchase_price,
                  credits_threshold = EXCLUDED.credits_threshold,
                  range_min = EXCLUDED.range_min,
                  cargo_min = EXCLUDED.cargo_min,
                  survey = EXCLUDED.survey,
                  extractor = EXCLUDED.extractor,
                  siphon = EXCLUDED.siphon,
                  warp_drive = EXCLUDED.warp_drive;
            "#,
            &item.id,
            &item.fleet_id,
            &item.priority,
            &item.max_purchase_price,
            &item.credits_threshold,
            &item.disabled,
            &item.range_min,
            &item.cargo_min,
            &item.survey,
            &item.extractor,
            &item.siphon,
            &item.warp_drive,
        )
        .execute(&database_pool.database_pool)
        .await?;
        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool, items))]
    async fn insert_bulk(database_pool: &DbPool, items: &[ShipAssignment]) -> crate::Result<()> {
        let (
            ids,
            fleet_ids,
            priority_values,
            max_purchase_price_values,
            credits_threshold_values,
            disabled_values,
            range_min_values,
            cargo_min_values,
            survey_values,
            extractor_values,
            siphon_values,
            warp_drive_values,
        ): (
            Vec<_>,
            Vec<_>,
            Vec<_>,
            Vec<_>,
            Vec<_>,
            Vec<_>,
            Vec<_>,
            Vec<_>,
            Vec<_>,
            Vec<_>,
            Vec<_>,
            Vec<_>,
        ) = itertools::multiunzip(items.iter().map(|sa| {
            (
                sa.id,
                sa.fleet_id,
                sa.priority,
                sa.max_purchase_price,
                sa.credits_threshold,
                sa.disabled,
                sa.range_min,
                sa.cargo_min,
                sa.survey,
                sa.extractor,
                sa.siphon,
                sa.warp_drive,
            )
        }));

        sqlx::query!(
            r#"
            INSERT INTO ship_assignment (
                id,
                fleet_id,
                priority,
                max_purchase_price,
                credits_threshold,
                disabled,
                range_min,
                cargo_min,
                survey,
                extractor,
                siphon,
                warp_drive
            )
            SELECT
                id,
                fid,
                pr,
                mxpp,
                ct,
                dis,
                rm,
                cm,
                sur,
                ext,
                sip,
                wd
            FROM UNNEST(
                $1::bigint[],
                $2::integer[],
                $3::integer[],
                $4::integer[],
                $5::integer[],
                $6::boolean[],
                $7::integer[],
                $8::integer[],
                $9::boolean[],
                $10::boolean[],
                $11::boolean[],
                $12::boolean[]
            ) AS t(id, fid, pr, mxpp, ct, dis, rm, cm, sur, ext, sip, wd)
            ON CONFLICT (id) DO UPDATE
            SET fleet_id = EXCLUDED.fleet_id,
                disabled = EXCLUDED.disabled,
                max_purchase_price = EXCLUDED.max_purchase_price,
                credits_threshold = EXCLUDED.credits_threshold,
                priority = EXCLUDED.priority,
                range_min = EXCLUDED.range_min,
                cargo_min = EXCLUDED.cargo_min,
                survey = EXCLUDED.survey,
                extractor = EXCLUDED.extractor,
                siphon = EXCLUDED.siphon,
                warp_drive = EXCLUDED.warp_drive;
            "#,
            &ids,
            &fleet_ids,
            &priority_values,
            &max_purchase_price_values,
            &credits_threshold_values,
            &disabled_values,
            &range_min_values,
            &cargo_min_values,
            &survey_values,
            &extractor_values,
            &siphon_values,
            &warp_drive_values,
        )
        .execute(&database_pool.database_pool)
        .await?;
        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn get_all(database_pool: &DbPool) -> crate::Result<Vec<ShipAssignment>> {
        let result = sqlx::query_as!(
            ShipAssignment,
            r#"
                SELECT
                  id,
                  fleet_id,
                  priority,
                  max_purchase_price,
                  credits_threshold,
                  disabled,
                  range_min,
                  cargo_min,
                  survey,
                  extractor,
                  siphon,
                  warp_drive
                FROM ship_assignment
            "#
        )
        .fetch_all(database_pool.get_cache_pool())
        .await?;
        Ok(result)
    }
}
