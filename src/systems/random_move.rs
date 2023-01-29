use crate::prelude::*;

#[system]
#[read_component(Point)]
#[read_component(MovingRandomly)]
#[read_component(Health)]
#[read_component(Player)]

pub fn random_moves(ecs: &SubWorld, commands: &mut CommandBuffer) {
    let mut movers = <(Entity, &Point)>::query().filter(component::<MovingRandomly>());
    let mut positions = <(Entity, &Point)>::query().filter(component::<Health>());

    movers.iter(ecs).for_each(|(entity, pos)| {
        let mut rng = RandomNumberGenerator::new();
        let destination = match rng.range(0, 4) {
            0 => Point::new(-1, 0),
            1 => Point::new(1, 0),
            2 => Point::new(0, -1),
            _ => Point::new(0, 1),
        } + *pos;

        let mut attacked = false;
        positions
            .iter(ecs)
            .filter(|(_, target_pos)| **target_pos == destination)
            .for_each(|(victim, _)| {
                if ecs
                    .entry_ref(*victim)
                    .unwrap()
                    .get_component::<Player>()
                    .is_ok()
                {
                    commands.push((
                        (),
                        WantsToAttack {
                            attacker: *entity,
                            victim: *victim,
                        },
                    ));
                }
                attacked = true;
            });
        if !attacked {
            commands.push((
                (),
                WantsToMove {
                    entity: *entity,
                    destination,
                },
            ));
        }
    })
}
