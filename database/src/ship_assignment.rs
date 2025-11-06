use tracing::instrument;

use crate::{DatabaseConnector, DbPool};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShipAssignment {
    pub id: i64,
    pub fleet_id: i32,
    pub priority: i32, // lower numbers are higher priority
    pub max_purchase_price: i32,
    pub credits_threshold: i32,
    pub disabled: bool, // if true, ship should not be assigned to this assignment and currently assigned ships should be removed
    pub range_min: i32, // aka fuel capacity minimum, -1 means infinite
    pub cargo_min: i32,
    pub survey: bool,
    pub extractor: bool,
    pub siphon: bool,
    pub warp_drive: bool,
    // pub refinery: bool,
}

impl ShipAssignment {
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
}

impl DatabaseConnector<ShipAssignment> for ShipAssignment {
    #[instrument(level = "trace", skip(database_pool))]
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

    #[instrument(level = "trace", skip(database_pool))]
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
