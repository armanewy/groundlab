use serde::{Deserialize, Serialize};

use crate::assault::{build_assault_influence, build_rolling_hazard_summary, summarize_assault};
use crate::{
    AssaultInfluenceSummary, AssaultSummary, MissionState, RollingHazardImpactSummary,
    RoutePredictionAccuracyReport,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MissionRating {
    pub stars: u8,
    pub label: String,
    pub objective_survived: bool,
    pub stopped_ratio: f32,
    pub objective_health_ratio: f32,
    pub prep_time_used_seconds: u32,
    pub prep_time_efficiency: f32,
    pub friendly_risk_count: u32,
    pub unused_defense_count: u32,
    pub hazard_enemies_hit: u32,
    pub score: i32,
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MissionBalanceScenarioReport {
    pub id: String,
    pub label: String,
    pub order_count: u32,
    pub prep_time_used_seconds: u32,
    pub summary: AssaultSummary,
    pub rating: MissionRating,
    pub route_prediction_accuracy: RoutePredictionAccuracyReport,
    pub rolling_hazards: RollingHazardImpactSummary,
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MissionBalanceReport {
    pub mission_id: String,
    pub mission_title: String,
    pub scenarios: Vec<MissionBalanceScenarioReport>,
    pub route_shift_summary: Vec<String>,
    pub hazard_effectiveness: Vec<String>,
    pub rating_breakdown: Vec<String>,
}

pub fn mission_rating_for_state(state: &MissionState) -> Option<MissionRating> {
    let assault = state.assault.as_ref()?;
    let summary = assault
        .summary
        .clone()
        .unwrap_or_else(|| summarize_assault(&state.spec, assault));
    let influence = build_assault_influence(state, assault);
    let rolling_hazards = build_rolling_hazard_summary(state, assault);
    Some(rate_mission_outcome(
        state,
        &summary,
        &influence,
        &rolling_hazards,
    ))
}

pub(crate) fn rate_mission_outcome(
    state: &MissionState,
    summary: &AssaultSummary,
    influence: &AssaultInfluenceSummary,
    rolling_hazards: &RollingHazardImpactSummary,
) -> MissionRating {
    let objective_survived = summary.victory;
    let stopped_ratio = if summary.enemies_spawned == 0 {
        1.0
    } else {
        summary.enemies_eliminated as f32 / summary.enemies_spawned as f32
    };
    let objective_health_ratio = if state.spec.objective.objective_health == 0 {
        0.0
    } else {
        summary.objective_health_remaining.max(0) as f32
            / state.spec.objective.objective_health as f32
    };
    let prep_time_used_seconds = state
        .spec
        .prep_time_seconds
        .saturating_sub(state.remaining_prep_seconds);
    let prep_time_efficiency = if state.spec.prep_time_seconds == 0 {
        0.0
    } else {
        state.remaining_prep_seconds as f32 / state.spec.prep_time_seconds as f32
    };
    let friendly_risk_count = rolling_hazards.friendly_risk_cells.len() as u32;
    let unused_defense_count = influence.unused_defenses.len() as u32;
    let hazard_enemies_hit = rolling_hazards.enemies_hit;

    let mut score = 0;
    if objective_survived {
        score += 50;
    }
    score += (stopped_ratio * 25.0).round() as i32;
    score += (objective_health_ratio * 15.0).round() as i32;
    score += (prep_time_efficiency * 10.0).round() as i32;
    score += hazard_enemies_hit.min(10) as i32;
    score -= friendly_risk_count as i32 * 10;
    score -= unused_defense_count.min(4) as i32 * 2;
    score = score.max(0);

    let stars = if !objective_survived {
        0
    } else if objective_health_ratio >= 0.90
        && stopped_ratio >= 0.85
        && prep_time_used_seconds <= 360
        && friendly_risk_count == 0
    {
        3
    } else if objective_health_ratio >= 0.70 && stopped_ratio >= 0.70 {
        2
    } else {
        1
    };
    let label = match stars {
        3 => "Decisive defense",
        2 => "Solid defense",
        1 => "Objective held",
        _ => "Objective lost",
    }
    .to_string();

    let mut notes = Vec::new();
    notes.push(if objective_survived {
        format!(
            "Objective survived with {:.0}% health.",
            objective_health_ratio * 100.0
        )
    } else {
        "Objective was overrun.".to_string()
    });
    notes.push(format!(
        "Stopped {:.0}% of attackers ({} of {}).",
        stopped_ratio * 100.0,
        summary.enemies_eliminated,
        summary.enemies_spawned
    ));
    notes.push(format!(
        "Prep used {}s of {}s.",
        prep_time_used_seconds, state.spec.prep_time_seconds
    ));
    if friendly_risk_count > 0 {
        notes.push(format!(
            "{friendly_risk_count} friendly-risk hazard cell(s) were flagged."
        ));
    }
    if unused_defense_count > 0 {
        notes.push(format!(
            "{unused_defense_count} prepared defense(s) did not affect enemy paths."
        ));
    }
    if hazard_enemies_hit > 0 {
        notes.push(format!(
            "Rolling hazards hit {hazard_enemies_hit} enemy agent(s)."
        ));
    }

    MissionRating {
        stars,
        label,
        objective_survived,
        stopped_ratio,
        objective_health_ratio,
        prep_time_used_seconds,
        prep_time_efficiency,
        friendly_risk_count,
        unused_defense_count,
        hazard_enemies_hit,
        score,
        notes,
    }
}
