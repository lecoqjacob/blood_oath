use bo_saveload::BoxedError;

use super::{ModeControl, ModeResult, *};

pub const MAIN_MENU_SCREEN_WIDTH: usize = 80;
pub const MAIN_MENU_SCREEN_HEIGHT: usize = 31;

#[derive(Debug)]
pub enum GameOverModeResult {
    AppQuit,
}

#[derive(Debug)]
pub enum MenuAction {
    Exit,
}

#[derive(Debug, Default)]
pub struct GameOverMode {
    selection: usize,
    actions: Vec<MenuAction>,
}

/// Show the title screen of the game with a menu that leads into the game proper.
impl GameOverMode {
    pub fn new() -> Self {
        let actions = vec![MenuAction::Exit];
        Self { actions, selection: 0 }
    }

    pub fn tick(
        &mut self,
        ctx: &mut BTerm,
        world: &mut World,
        _pop_result: &Option<ModeResult>,
    ) -> (ModeControl, ModeUpdate) {
        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::Escape => {
                    return (ModeControl::Pop(GameOverModeResult::AppQuit.into()), ModeUpdate::Immediate)
                }
                VirtualKeyCode::Return => {
                    assert!(self.selection < self.actions.len());

                    if let Err(e) = self.game_over_cleanup(world) {
                        eprintln!("Warning: game_over_cleanup error: {}", e);
                    }

                    return (ModeControl::Switch(MainMenuMode::new().into()), ModeUpdate::Immediate);
                }
                _ => {}
            }
        }

        (ModeControl::Stay, ModeUpdate::Update)
    }

    pub fn draw(&self, _ctx: &mut BTerm, _world: &World, _active: bool) {
        let mut draw_batch = DrawBatch::new();

        draw_batch.print_color_centered(15, "Your journey has ended!", ColorPair::new(YELLOW, BLACK));
        draw_batch.print_color_centered(
            17,
            "One day, we'll tell you all about how you did.",
            ColorPair::new(WHITE, BLACK),
        );
        draw_batch.print_color_centered(
            18,
            "That day, sadly, is not in this chapter..",
            ColorPair::new(WHITE, BLACK),
        );

        draw_batch.print_color_centered(
            19,
            &format!("You lived for {} turns.", bo_logging::get_event_count(TURN_DONE_EVENT)),
            ColorPair::new(WHITE, BLACK),
        );
        draw_batch.print_color_centered(
            20,
            &format!("You suffered {} points of damage.", bo_logging::get_event_count(DAMAGE_TAKE_EVENT)),
            ColorPair::new(RED, BLACK),
        );
        draw_batch.print_color_centered(
            21,
            &format!("You inflicted {} points of damage.", bo_logging::get_event_count(DAMAGE_INFLICT_EVENT)),
            ColorPair::new(RED, BLACK),
        );

        draw_batch.print_color_centered(
            23,
            "Press any key to return to the menu.",
            ColorPair::new(MAGENTA, BLACK),
        );

        draw_batch.submit(BATCH_ZERO).expect("Error batching title");
    }
}

impl GameOverMode {
    fn game_over_cleanup(&mut self, world: &mut World) -> Result<(), BoxedError> {
        // Delete everything
        #[cfg(target_arch = "wasm32")]
        let to_delete = world.entities().join().collect::<Vec<_>>();

        #[cfg(not(target_arch = "wasm32"))]
        let to_delete = world.entities().par_join().collect::<Vec<_>>();

        // Delete all Entities
        world.delete_entities(&to_delete)?;

        Ok(())
    }
}
