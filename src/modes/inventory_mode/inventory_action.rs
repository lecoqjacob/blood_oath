use super::*;

const CANCEL: &str = "[ Cancel ]";

#[derive(Debug)]
pub enum InventoryActionModeResult {
    AppQuit,
    Cancelled,
    DropItem(Entity),
    // EquipItem(Entity),
    UseItem(Entity),
}

#[derive(Debug)]
enum SubSection {
    Cancel,
    Actions,
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum InventoryAction {
    // EquipItem,
    UseItem,
    DropItem,
}

impl InventoryAction {
    pub fn from_key(key: VirtualKeyCode) -> Option<Self> {
        match key {
            VirtualKeyCode::A => Some(InventoryAction::UseItem),
            VirtualKeyCode::D => Some(InventoryAction::DropItem),
            _ => None,
        }
    }

    pub fn item_supports_action(world: &World, item: Entity, action: InventoryAction) -> bool {
        match action {
            // InventoryAction::EquipItem => world.borrow::<View<EquipSlot>>().contains(item),
            InventoryAction::DropItem => true,
            InventoryAction::UseItem => world.read_storage::<Consumable>().contains(item),
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            // InventoryAction::EquipItem => "Equip",
            InventoryAction::UseItem => "Apply",
            InventoryAction::DropItem => "Drop",
        }
    }

    fn label(&self) -> &'static str {
        match self {
            // InventoryAction::EquipItem => "[ Equip ]",
            InventoryAction::UseItem => "[ Apply ]",
            InventoryAction::DropItem => "[ Drop ]",
        }
    }
}

#[derive(Debug)]
pub struct InventoryActionMode {
    item_id: Entity,
    selection: usize,
    inner_width: i32,
    subsection: SubSection,
    item_desc: (Glyph, String),
    actions: Vec<InventoryAction>,
}

/// Show a menu of actions for a single item in the player's inventory.
impl InventoryActionMode {
    pub fn new(world: &World, item_id: Entity, default_action: Option<InventoryAction>) -> Self {
        let actions = [InventoryAction::UseItem, InventoryAction::DropItem]
            .iter()
            .filter(|action| InventoryAction::item_supports_action(world, item_id, **action))
            .copied()
            .collect::<Vec<_>>();

        let selection =
            default_action.and_then(|d_act| actions.iter().position(|a| *a == d_act)).unwrap_or(0);
        let subsection = if actions.is_empty() { SubSection::Cancel } else { SubSection::Actions };

        let item_width = world.read_storage::<Name>().get(item_id).unwrap().0.len();
        let inner_width =
            3 + item_width.max(CANCEL.len()).max(actions.iter().map(|a| a.label().len()).max().unwrap_or(0))
                as i32;

        let item_glyph = world.read_storage::<Glyph>().get(item_id).unwrap().clone();
        let item_name = world.read_storage::<Name>().get(item_id).unwrap().0.clone();

        Self {
            item_id,
            actions,
            subsection,
            selection: selection,
            inner_width,
            item_desc: (item_glyph, item_name),
        }
    }

    fn confirm_action(&self, world: &World) -> ModeControl {
        let result = match self.subsection {
            SubSection::Cancel => InventoryActionModeResult::Cancelled,
            SubSection::Actions => match self.actions[self.selection as usize] {
                InventoryAction::DropItem => InventoryActionModeResult::DropItem(self.item_id),
                InventoryAction::UseItem => {
                    if let Some(ranged) = world.read_storage::<Ranged>().get(self.item_id) {
                        let item_name = world.read_storage::<Name>().get(self.item_id).unwrap().0.clone();

                        let radius = world
                            .read_storage::<AreaOfEffect>()
                            .get(self.item_id)
                            .map_or(0, |aoe| aoe.radius);

                        // return ModeControl::Pop(
                        //     TargetingMode::new(world, item_name, *range, radius, true).into(),
                        // );
                        // return ModeControl::Pop(InventoryActionModeResult::Targeting().into());
                        // return InventoryActionModeResult::UseItem(self.item_id);
                    }

                    InventoryActionModeResult::UseItem(self.item_id)
                }
            },
        };

        ModeControl::Pop(result.into())
    }

    pub fn tick(
        &mut self,
        ctx: &mut BTerm,
        world: &mut World,
        _pop_result: &Option<ModeResult>,
    ) -> ModeControl {
        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::Escape => {
                    return ModeControl::Pop(InventoryActionModeResult::Cancelled.into())
                }
                VirtualKeyCode::Down => match self.subsection {
                    SubSection::Actions => {
                        if self.selection < self.actions.len() - 1 {
                            self.selection += 1;
                        } else {
                            self.subsection = SubSection::Cancel;
                        }
                    }
                    SubSection::Cancel => {
                        if !self.actions.is_empty() {
                            self.subsection = SubSection::Actions;
                            self.selection = 0;
                        }
                    }
                },
                VirtualKeyCode::Up => match self.subsection {
                    SubSection::Actions => {
                        if self.selection > 0 {
                            self.selection -= 1;
                        } else {
                            self.subsection = SubSection::Cancel;
                        }
                    }
                    SubSection::Cancel => {
                        if !self.actions.is_empty() {
                            self.subsection = SubSection::Actions;
                            self.selection = self.actions.len() - 1;
                        }
                    }
                },
                VirtualKeyCode::Return => {
                    return self.confirm_action(world);
                }

                key @ VirtualKeyCode::D | key @ VirtualKeyCode::A => {
                    if let Some(inv_action) = InventoryAction::from_key(key) {
                        if let Some(action_pos) = self.actions.iter().position(|a| *a == inv_action) {
                            if matches!(self.subsection, SubSection::Actions) && self.selection == action_pos
                            {
                                return self.confirm_action(world);
                            } else {
                                self.subsection = SubSection::Actions;
                                self.selection = action_pos;
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        ModeControl::Stay
    }

    pub fn draw(&self, _ctx: &mut BTerm, _world: &mut World, _active: bool) {
        let mut draw_batch = DrawBatch::new();
        draw_batch.target(0);

        let box_rect = center_box(
            &mut draw_batch,
            (SCREEN_WIDTH, SCREEN_HEIGHT),
            BoxConfig::new((self.inner_width, 10), ColorPair::new(WHITE, BLACK), true, false),
        );

        let x = box_rect.x1 + 1;
        let mut y = box_rect.y1 + 1;
        let (item_glyph, item_name) = &self.item_desc;

        draw_batch.set(Point::new(x, y), item_glyph.color, item_glyph.glyph);
        draw_batch.print(Point::new(x + 2, y), item_name);

        y += 2;
        for (i, action) in self.actions.iter().enumerate() {
            let bg = if matches!(self.subsection, SubSection::Actions) && i == self.selection {
                bo_utils::SELECTED_BG
            } else {
                BLACK
            };

            draw_batch.print_color_centered_at(
                Point::new(x + box_rect.width() / 2, y + i as i32),
                action.label(),
                ColorPair::new(WHITE, bg),
            );
        }

        draw_batch.print_color_centered_at(
            Point::new(x + box_rect.width() / 2, y + 3),
            CANCEL,
            ColorPair::new(
                WHITE,
                if matches!(self.subsection, SubSection::Cancel) { bo_utils::SELECTED_BG } else { BLACK },
            ),
        );

        draw_batch.submit(BATCH_UI_INV + 1000).expect("Batch error"); // On top of everything
    }
}