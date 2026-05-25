use crate::{
    mission_visual_theme_for_theme, CellCoord, CellRect, CoverClass, CrewPool,
    DefenderPositionSpec, Direction, EnemyDoctrine, EnemyGroupSpec, EnvironmentObject,
    EnvironmentObjectKind, GroundKind, LogState, MissionBriefing, MissionCell, MissionConstraints,
    MissionMap, MissionObjective, MissionSpec, MissionTheme, MovementProfile, RockState,
    ScriptedWorkOrder, ToolLoadout, TreeState, WorkOrderKind, WorkOrderScript, WorkTarget,
};

pub fn road_below_spec() -> MissionSpec {
    let mut map = MissionMap::new(12, 8, MissionCell::new(1, GroundKind::Grass));
    map.spawn_cells.push(CellCoord::new(1, 7));
    map.spawn_cells.push(CellCoord::new(0, 6));

    for x in 0..map.width {
        let cell = CellCoord::new(x, 4);
        let Some(tile) = map.cell_mut(cell) else {
            continue;
        };
        tile.ground = GroundKind::Road;
        tile.movement_cost = GroundKind::Road.base_movement_cost();
    }

    for x in 7..=11 {
        if let Some(tile) = map.cell_mut(CellCoord::new(x, 2)) {
            tile.height = 2;
        }
        if let Some(tile) = map.cell_mut(CellCoord::new(x, 3)) {
            tile.height = 2;
        }
    }

    for (id, label, cell) in [
        ("tree_west_01", "roadside pine", CellCoord::new(3, 2)),
        ("tree_west_02", "screening pine", CellCoord::new(4, 2)),
        ("tree_east_01", "low orchard tree", CellCoord::new(8, 5)),
    ] {
        map.objects.push(EnvironmentObject {
            id: id.to_string(),
            label: label.to_string(),
            kind: EnvironmentObjectKind::Tree(TreeState::Standing),
            cell,
            footprint: (1, 1),
            blocks_sight: true,
            cover: CoverClass::Partial,
            movement_cost_delta: 0.4,
        });
    }

    map.objects.push(EnvironmentObject {
        id: "ridge_stone_01".to_string(),
        label: "loose ridge stone".to_string(),
        kind: EnvironmentObjectKind::Rock(RockState::Stable),
        cell: CellCoord::new(8, 2),
        footprint: (1, 1),
        blocks_sight: false,
        cover: CoverClass::Light,
        movement_cost_delta: 0.3,
    });

    map.objects.push(EnvironmentObject {
        id: "ridge_log_01".to_string(),
        label: "ridge rolling log".to_string(),
        kind: EnvironmentObjectKind::Log(LogState::Loose {
            direction: Direction::South,
        }),
        cell: CellCoord::new(7, 3),
        footprint: (2, 1),
        blocks_sight: false,
        cover: CoverClass::Light,
        movement_cost_delta: 0.8,
    });

    MissionSpec {
        id: "road_below".to_string(),
        title: "The Road Below".to_string(),
        briefing: MissionBriefing {
            summary: "A southern road climbs toward a low ridge marker. Use limited prep time to shape the approach before the assault.".to_string(),
            primary: "Keep the ridge marker intact until the attack breaks.".to_string(),
            optional_objectives: vec![
                "Stop at least 70% of attackers.".to_string(),
                "Spend less than six minutes of prep time.".to_string(),
                "Avoid friendly-risk hazard paths.".to_string(),
            ],
            intel: vec![
                "Rushers favor the road and shortest climb.".to_string(),
                "Skirmishers look for cover and concealment on the orchard side.".to_string(),
                "The ridge log can punish bunching but crosses a friendly-risk cell.".to_string(),
            ],
        },
        visual_theme: mission_visual_theme_for_theme(MissionTheme::DryRoadBelow),
        objective: MissionObjective {
            label: "Hold the ridge marker".to_string(),
            defend_cell: CellCoord::new(10, 3),
            objective_health: 100,
        },
        prep_time_seconds: 480,
        map,
        starting_tools: ToolLoadout::basic_field_kit(),
        crew: CrewPool {
            crews: 3,
            labor_seconds_available: 480,
        },
        enemy_groups: vec![
            EnemyGroupSpec {
                label: "southern rushers".to_string(),
                count: 12,
                doctrine: EnemyDoctrine::RushShortest,
                spawn: CellCoord::new(1, 7),
                objective: CellCoord::new(10, 3),
                movement_profile: MovementProfile {
                    base_speed: 1.0,
                    obstacle_tolerance: 0.35,
                    cover_preference: 0.1,
                },
            },
            EnemyGroupSpec {
                label: "cautious riflemen".to_string(),
                count: 8,
                doctrine: EnemyDoctrine::PreferCover,
                spawn: CellCoord::new(0, 6),
                objective: CellCoord::new(10, 3),
                movement_profile: MovementProfile {
                    base_speed: 0.85,
                    obstacle_tolerance: 0.2,
                    cover_preference: 0.65,
                },
            },
            EnemyGroupSpec {
                label: "orchard skirmishers".to_string(),
                count: 6,
                doctrine: EnemyDoctrine::FlankViaConcealment,
                spawn: CellCoord::new(0, 6),
                objective: CellCoord::new(10, 3),
                movement_profile: MovementProfile {
                    base_speed: 0.95,
                    obstacle_tolerance: 0.3,
                    cover_preference: 0.45,
                },
            },
        ],
        defender_positions: vec![
            DefenderPositionSpec {
                id: "ridge_team_01".to_string(),
                label: "ridge rifle pit".to_string(),
                cell: CellCoord::new(9, 3),
                range: 5,
                pressure_per_step: 2,
            },
            DefenderPositionSpec {
                id: "road_team_01".to_string(),
                label: "road overwatch".to_string(),
                cell: CellCoord::new(7, 3),
                range: 4,
                pressure_per_step: 1,
            },
        ],
        constraints: MissionConstraints {
            max_work_orders: 12,
            allow_assault_preview: false,
        },
    }
}

pub fn road_below_seed_orders() -> Vec<(WorkOrderKind, WorkTarget)> {
    vec![
        (
            WorkOrderKind::DigTrench,
            WorkTarget::Rect(CellRect {
                origin: CellCoord::new(5, 4),
                width: 2,
                height: 1,
            }),
        ),
        (
            WorkOrderKind::RaiseBerm,
            WorkTarget::Rect(CellRect {
                origin: CellCoord::new(5, 3),
                width: 2,
                height: 1,
            }),
        ),
        (
            WorkOrderKind::FellTree,
            WorkTarget::Object("tree_west_01".to_string()),
        ),
        (
            WorkOrderKind::CutIntoLogs,
            WorkTarget::Object("tree_west_01".to_string()),
        ),
        (
            WorkOrderKind::PlaceStakes,
            WorkTarget::Cell(CellCoord::new(3, 4)),
        ),
    ]
}

pub fn road_below_basic_prep_script() -> WorkOrderScript {
    WorkOrderScript {
        id: "road_below_basic_prep".to_string(),
        label: "Road Below basic prep".to_string(),
        orders: road_below_seed_orders()
            .into_iter()
            .map(|(kind, target)| ScriptedWorkOrder { kind, target })
            .collect(),
    }
}

pub fn road_below_hazard_prep_script() -> WorkOrderScript {
    WorkOrderScript {
        id: "road_below_hazard_prep".to_string(),
        label: "Road Below rolling hazard prep".to_string(),
        orders: vec![ScriptedWorkOrder {
            kind: WorkOrderKind::PrepareRollingLog,
            target: WorkTarget::Object("ridge_log_01".to_string()),
        }],
    }
}

pub fn road_below_no_prep_script() -> WorkOrderScript {
    WorkOrderScript {
        id: "baseline_no_prep".to_string(),
        label: "Baseline: no prep".to_string(),
        orders: Vec::new(),
    }
}

pub fn road_below_trench_line_script() -> WorkOrderScript {
    WorkOrderScript {
        id: "basic_trench_line".to_string(),
        label: "Basic trench line".to_string(),
        orders: vec![ScriptedWorkOrder {
            kind: WorkOrderKind::DigTrench,
            target: WorkTarget::Rect(CellRect {
                origin: CellCoord::new(5, 4),
                width: 2,
                height: 1,
            }),
        }],
    }
}

pub fn road_below_berm_and_stakes_script() -> WorkOrderScript {
    WorkOrderScript {
        id: "berm_and_stakes".to_string(),
        label: "Berm and stakes".to_string(),
        orders: vec![
            ScriptedWorkOrder {
                kind: WorkOrderKind::DigTrench,
                target: WorkTarget::Rect(CellRect {
                    origin: CellCoord::new(5, 4),
                    width: 2,
                    height: 1,
                }),
            },
            ScriptedWorkOrder {
                kind: WorkOrderKind::RaiseBerm,
                target: WorkTarget::Rect(CellRect {
                    origin: CellCoord::new(5, 3),
                    width: 2,
                    height: 1,
                }),
            },
            ScriptedWorkOrder {
                kind: WorkOrderKind::FellTree,
                target: WorkTarget::Object("tree_west_01".to_string()),
            },
            ScriptedWorkOrder {
                kind: WorkOrderKind::CutIntoLogs,
                target: WorkTarget::Object("tree_west_01".to_string()),
            },
            ScriptedWorkOrder {
                kind: WorkOrderKind::PlaceStakes,
                target: WorkTarget::Cell(CellCoord::new(3, 4)),
            },
        ],
    }
}

pub fn road_below_ridge_chokepoint_script() -> WorkOrderScript {
    WorkOrderScript {
        id: "ridge_chokepoint".to_string(),
        label: "Ridge chokepoint".to_string(),
        orders: vec![
            ScriptedWorkOrder {
                kind: WorkOrderKind::FellTree,
                target: WorkTarget::Object("tree_west_02".to_string()),
            },
            ScriptedWorkOrder {
                kind: WorkOrderKind::CutIntoLogs,
                target: WorkTarget::Object("tree_west_02".to_string()),
            },
            ScriptedWorkOrder {
                kind: WorkOrderKind::DigTrench,
                target: WorkTarget::Rect(CellRect {
                    origin: CellCoord::new(2, 6),
                    width: 2,
                    height: 1,
                }),
            },
            ScriptedWorkOrder {
                kind: WorkOrderKind::RaiseBerm,
                target: WorkTarget::Rect(CellRect {
                    origin: CellCoord::new(2, 5),
                    width: 2,
                    height: 1,
                }),
            },
            ScriptedWorkOrder {
                kind: WorkOrderKind::PlaceStakes,
                target: WorkTarget::Cell(CellCoord::new(4, 5)),
            },
        ],
    }
}

pub fn road_below_overbuilt_bad_plan_script() -> WorkOrderScript {
    let mut orders = road_below_seed_orders()
        .into_iter()
        .map(|(kind, target)| ScriptedWorkOrder { kind, target })
        .collect::<Vec<_>>();
    orders.push(ScriptedWorkOrder {
        kind: WorkOrderKind::FellTree,
        target: WorkTarget::Object("tree_east_01".to_string()),
    });
    WorkOrderScript {
        id: "overbuilt_bad_plan".to_string(),
        label: "Overbuilt bad plan".to_string(),
        orders,
    }
}

pub fn road_below_balance_scripts() -> Vec<WorkOrderScript> {
    vec![
        road_below_no_prep_script(),
        road_below_trench_line_script(),
        road_below_berm_and_stakes_script(),
        road_below_basic_prep_script(),
        road_below_hazard_prep_script(),
        road_below_ridge_chokepoint_script(),
        road_below_overbuilt_bad_plan_script(),
    ]
}
