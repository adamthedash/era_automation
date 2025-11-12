use crate::crafting::Recipe;
use crate::items::ItemType;
use crate::knowledge::UnlockRequirement;
use crate::resources::ResourceType;

/// A single knowledge definition .
#[derive(Debug)]
pub struct KnowledgeDef {
    pub name: String,
    pub requirements: Vec<UnlockRequirement>,
    pub recipe: Option<Recipe>,
}

impl KnowledgeDef {
    /// Create a new knowledge definition with the given `name` and an empty
    /// requirements list.
    pub fn new(name: impl Into<String>) -> Self {
        KnowledgeDef {
            name: name.into(),
            requirements: Vec::new(),
            recipe: None,
        }
    }

    /// Add a requirement to this definition.
    pub fn requirement(mut self, req: UnlockRequirement) -> Self {
        self.requirements.push(req);
        self
    }

    /// Set the optional recipe for this knowledge definition.
    pub fn recipe(mut self, recipe: Recipe) -> Self {
        self.recipe = Some(recipe);
        self
    }
}

/// Load the game's knowledge definitions.
pub fn load_knowledge() -> Vec<KnowledgeDef> {
    vec![
        KnowledgeDef::new("Bowl")
            .requirement(UnlockRequirement::TotalDeposited {
                resource: ResourceType::Wood,
                amount: 1,
            })
            .requirement(UnlockRequirement::TotalDeposited {
                resource: ResourceType::Water,
                amount: 1,
            })
            .recipe(Recipe {
                reqs: vec![(ResourceType::Wood, 5)],
                product: ItemType::Bowl,
            }),
        KnowledgeDef::new("Harvester")
            .requirement(UnlockRequirement::TotalDeposited {
                resource: ResourceType::Wood,
                amount: 0,
            })
            .requirement(UnlockRequirement::TotalDeposited {
                resource: ResourceType::Food,
                amount: 0,
            })
            .recipe(Recipe {
                reqs: vec![(ResourceType::Wood, 5)],
                product: ItemType::BushWhacker,
            }),
        KnowledgeDef::new("Transporter")
            .requirement(UnlockRequirement::TotalRolled {
                item: ItemType::Log,
                distance: 10.0,
            })
            .recipe(Recipe {
                reqs: vec![(ResourceType::Wood, 5)],
                product: ItemType::Transporter,
            }),
        KnowledgeDef::new("Picker-upper")
            // no requirements
            .recipe(Recipe {
                reqs: vec![(ResourceType::Wood, 5)],
                product: ItemType::PickerUpper,
            }),
        KnowledgeDef::new("Trip Axe")
            .requirement(UnlockRequirement::TotalDeposited {
                resource: ResourceType::Wood,
                amount: 0,
            })
            .requirement(UnlockRequirement::TotalDeposited {
                resource: ResourceType::Food,
                amount: 0,
            })
            .recipe(Recipe {
                reqs: vec![(ResourceType::Wood, 5)],
                product: ItemType::TripAxe,
            }),
        KnowledgeDef::new("Water Wheel")
            .requirement(UnlockRequirement::TotalDeposited {
                resource: ResourceType::Wood,
                amount: 0,
            })
            .requirement(UnlockRequirement::TotalDeposited {
                resource: ResourceType::Food,
                amount: 0,
            })
            .recipe(Recipe {
                reqs: vec![(ResourceType::Wood, 5)],
                product: ItemType::WaterWheel,
            }),
        KnowledgeDef::new("Windmill")
            .requirement(UnlockRequirement::TotalDeposited {
                resource: ResourceType::Wood,
                amount: 1,
            })
            .recipe(Recipe {
                reqs: vec![(ResourceType::Wood, 1)],
                product: ItemType::Windmill,
            }),
        KnowledgeDef::new("Plant Watering")
            .requirement(UnlockRequirement::TotalDeposited {
                resource: ResourceType::Food,
                amount: 2,
            })
            .requirement(UnlockRequirement::TotalDeposited {
                resource: ResourceType::Water,
                amount: 2,
            }),
    ]
}
