use super::*;

pub struct MonsterAISystem {}

impl<'a> System<'a> for MonsterAISystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        Entities<'a>,
        ReadExpect<'a, Map>,
        ReadExpect<'a, Point>,
        ReadExpect<'a, Entity>,
        ReadExpect<'a, TurnState>,
        WriteStorage<'a, FieldOfView>,
        ReadStorage<'a, Monster>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, WantsToMelee>,
        WriteStorage<'a, Confusion>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            map,
            player_pos,
            player_entity,
            runstate,
            mut fov_storage,
            monster,
            mut position,
            mut wants_to_melee,
            mut confused,
        ) = data;

        if *runstate != TurnState::MonsterTurn {
            return;
        }

        for (entity, mut fov, _monster, mut pos) in
            (&entities, &mut fov_storage, &monster, &mut position).join()
        {
            let can_act = if let Some(i_am_confused) = confused.get_mut(entity) {
                i_am_confused.turns -= 1;

                if i_am_confused.turns < 1 {
                    confused.remove(entity);
                }

                false
            } else {
                true
            };

            if can_act {
                let distance = DistanceAlg::Pythagoras.distance2d(pos.0, *player_pos);
                if distance < 1.5 {
                    wants_to_melee
                        .insert(entity, WantsToMelee { target: *player_entity })
                        .expect("Unable to insert attack");
                } else if fov.visible_tiles.contains(&*player_pos) {
                    let old_idx = map.point2d_to_index(pos.0);
                    let new_idx = map.point2d_to_index(*player_pos);

                    // Path to the player
                    let path = a_star_search(old_idx, new_idx, &*map);

                    if path.success && path.steps.len() > 1 {
                        let destination = map.index_to_point2d(path.steps[1]);

                        crate::spatial::move_entity(
                            entity,
                            map.point2d_to_index(pos.0),
                            map.point2d_to_index(destination),
                        );

                        pos.0 = destination;
                        fov.is_dirty = true;
                    }
                }
            }
        }
    }
}
