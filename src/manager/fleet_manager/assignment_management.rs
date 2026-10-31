use database::{DatabaseConnectorAsync, ShipAssignment};
use itertools::Itertools;
use tracing::{debug, info};

use crate::utils::ConductorContext;

#[derive(Debug)]
pub struct AssignmentResult {
    pub merged_assignments: Vec<ShipAssignment>, // will be kept and updated
    pub non_merged_old_assignments: Vec<ShipAssignment>, // will need to be deleted
    pub non_merged_new_assignments: Vec<ShipAssignment>, // will need to be created
}

pub async fn update_fleet_assignments(
    context: &ConductorContext,
    result: AssignmentResult,
) -> crate::error::Result<()> {
    for mut assignment in result.non_merged_old_assignments {
        // database::ShipAssignment::delete_by_id(&context.database_pool, assignment.id).await?;
        // info!(assignment_id = assignment.id, "Deleted old ship assignment");
        assignment.disabled = true;
        database::ShipAssignment::upsert(&context.database_pool, &assignment).await?;
        info!(
            assignment_id = assignment.id,
            "Disabled old ship assignment"
        )
    }

    for assignment in result.non_merged_new_assignments {
        let created_assignment =
            database::ShipAssignment::insert_new(&context.database_pool, &assignment).await?;
        info!(
            assignment_id = created_assignment,
            "Created new ship assignment"
        );
    }

    for assignment in result.merged_assignments {
        database::ShipAssignment::upsert(&context.database_pool, &assignment).await?;
        info!(
            assignment_id = assignment.id,
            "Updated merged ship assignment"
        );
    }

    Ok(())
}

pub async fn fix_fleet_assignments(
    current_assignments: Vec<ShipAssignment>,
    new_assignments: Vec<ShipAssignment>,
) -> crate::error::Result<AssignmentResult> {
    // https://en.wikipedia.org/wiki/Bipartite_graph
    // https://en.wikipedia.org/wiki/Maximum_weight_matching
    // https://en.wikipedia.org/wiki/Hungarian_algorithm

    // if current_assignments.is_empty() {
    //     return Ok(AssignmentResult {
    //         merged_assignments: vec![],
    //         non_merged_old_assignments: vec![],
    //         non_merged_new_assignments: new_assignments,
    //     });
    // }

    // if current_assignments.len() == 1
    //     && new_assignments.len() == 1
    //     && current_assignments[0].can_merge(&new_assignments[0])
    // {
    //     let mut merged = current_assignments[0].clone();
    //     merged.merge_into(&new_assignments[0]);
    //     return Ok(AssignmentResult {
    //         merged_assignments: vec![merged],
    //         non_merged_old_assignments: vec![],
    //         non_merged_new_assignments: vec![],
    //     });
    // }

    let max_assignments = current_assignments.len().max(new_assignments.len());

    let weights = pathfinding::matrix::Matrix::from_rows(
        current_assignments
            .iter()
            .map(Some)
            .pad_using(max_assignments, |_| None)
            .map(|ca| {
                new_assignments
                    .iter()
                    .map(Some)
                    .pad_using(max_assignments, |_| None)
                    .map(|na| {
                        if na.is_none() || ca.is_none() {
                            return ordered_float::OrderedFloat(1_000_001.0);
                        }
                        let ca = ca.unwrap();
                        let na = na.unwrap();

                        let can_merge = ca.can_merge(na);
                        let score = if can_merge {
                            ca.merge_score(na)
                        } else {
                            1_000_000.0
                        };
                        // AssignmentMatch {
                        //     current_assignment_index: ca,
                        //     new_assignment_index: na,
                        //     score,
                        // }
                        ordered_float::OrderedFloat(score)
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<Vec<_>>>(),
    )
    .unwrap();

    let (cash_flow, assignments) = pathfinding::prelude::kuhn_munkres_min(&weights);

    debug!(
        ?cash_flow,
        ?assignments,
        ?current_assignments,
        ?new_assignments,
        ?weights,
        "Calculated optimal assignment matching"
    );

    let mut merged_assignments = vec![];
    let mut non_merged_old_assignments = vec![];
    let mut new_assignments = new_assignments
        .into_iter()
        .map(|f| (f, false))
        .collect::<Vec<_>>();

    for (assignment, new_assignment_index) in current_assignments.into_iter().zip(assignments) {
        let new_assignment = new_assignments.get_mut(new_assignment_index);
        if let Some((new_assignment, merged)) = new_assignment {
            if assignment.can_merge(new_assignment) {
                *merged = true;
                let mut merged = assignment.clone();
                merged.merge_into(new_assignment);
                merged_assignments.push(merged);
            } else {
                merged_assignments.push(assignment);
            }
        } else {
            non_merged_old_assignments.push(assignment);
        }
    }

    let non_merged_new_assignments = new_assignments
        .into_iter()
        .filter_map(|(assignment, merged)| if !merged { Some(assignment) } else { None })
        .collect::<Vec<_>>();

    Ok(AssignmentResult {
        merged_assignments,
        non_merged_old_assignments,
        non_merged_new_assignments,
    })
}
