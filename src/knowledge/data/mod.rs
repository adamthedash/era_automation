use crate::crafting::Recipe;
use crate::items::ItemType;
use crate::knowledge::UnlockRequirement;
use crate::resources::ResourceType;

pub struct KnowledgeDef {
    pub name: String,
    pub requirements: Vec<UnlockRequirement>,
    pub recipe: Option<Recipe>,
}

/// Load the game's knowledge definitions.
pub fn load_knowledge() -> Vec<KnowledgeDef> {
    vec![
        KnowledgeDef {
            name: "Bowl".to_string(),
            requirements: vec![
                UnlockRequirement::TotalDeposited {
                    resource: ResourceType::Wood,
                    amount: 1,
                },
                UnlockRequirement::TotalDeposited {
                    resource: ResourceType::Water,
                    amount: 1,
                },
            ],
            recipe: Some(Recipe {
                reqs: vec![(ResourceType::Wood, 5)],
                product: ItemType::Bowl,
            }),
        },
        KnowledgeDef {
            name: "Harvester".to_string(),
            requirements: vec![
                UnlockRequirement::TotalDeposited {
                    resource: ResourceType::Wood,
                    amount: 0,
                },
                UnlockRequirement::TotalDeposited {
                    resource: ResourceType::Food,
                    amount: 0,
                },
            ],
            recipe: Some(Recipe {
                reqs: vec![(ResourceType::Wood, 5)],
                product: ItemType::BushWhacker,
            }),
        },
        KnowledgeDef {
            name: "Transporter".to_string(),
            requirements: vec![UnlockRequirement::TotalRolled {
                item: ItemType::Log,
                distance: 10.0,
            }],
            recipe: Some(Recipe {
                reqs: vec![(ResourceType::Wood, 5)],
                product: ItemType::Transporter,
            }),
        },
        KnowledgeDef {
            name: "Picker-upper".to_string(),
            requirements: vec![],
            recipe: Some(Recipe {
                reqs: vec![(ResourceType::Wood, 5)],
                product: ItemType::PickerUpper,
            }),
        },
        KnowledgeDef {
            name: "Trip Axe".to_string(),
            requirements: vec![
                UnlockRequirement::TotalDeposited {
                    resource: ResourceType::Wood,
                    amount: 0,
                },
                UnlockRequirement::TotalDeposited {
                    resource: ResourceType::Food,
                    amount: 0,
                },
            ],
            recipe: Some(Recipe {
                reqs: vec![(ResourceType::Wood, 5)],
                product: ItemType::TripAxe,
            }),
        },
        KnowledgeDef {
            name: "Water Wheel".to_string(),
            requirements: vec![
                UnlockRequirement::TotalDeposited {
                    resource: ResourceType::Wood,
                    amount: 0,
                },
                UnlockRequirement::TotalDeposited {
                    resource: ResourceType::Food,
                    amount: 0,
                },
            ],
            recipe: Some(Recipe {
                reqs: vec![(ResourceType::Wood, 5)],
                product: ItemType::WaterWheel,
            }),
        },
        KnowledgeDef {
            name: "Plant Watering".to_string(),
            requirements: vec![
                UnlockRequirement::TotalDeposited {
                    resource: ResourceType::Food,
                    amount: 2,
                },
                UnlockRequirement::TotalDeposited {
                    resource: ResourceType::Water,
                    amount: 2,
                },
            ],
            recipe: None,
        },
    ]
}
