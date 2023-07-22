use super::{IUnitService, UnitService};
use crate::model::*;
use ahash::AHashSet;
use glam::*;
use uuid::Uuid;

#[derive(Default)]
pub struct GenerationService {
    init_flags: AHashSet<IVec2>,
}

impl GenerationService {
    pub fn generate(
        &mut self,
        aabb: IAabb3,
        iunit_service: &mut IUnitService,
        unit_service: &mut UnitService,
    ) {
        for x in aabb.min.x..=aabb.max.x {
            for y in aabb.min.y..=aabb.max.y {
                let pos = IVec2::new(x, y);
                if !self.init_flags.contains(&pos) {
                    // generation rules start

                    iunit_service
                        .add_iunit(IUnit::new(IVec3::new(x, y, 0), IUnitKind::SurfaceGrass));

                    if rand::random::<f32>() < 0.08 {
                        iunit_service
                            .add_iunit(IUnit::new(IVec3::new(x, y, 1), IUnitKind::MixGrass));
                    }

                    if rand::random::<f32>() < 0.02 {
                        iunit_service
                            .add_iunit(IUnit::new(IVec3::new(x, y, 1), IUnitKind::Dandelion));
                    }

                    if rand::random::<f32>() < 0.01 {
                        iunit_service
                            .add_iunit(IUnit::new(IVec3::new(x, y, 1), IUnitKind::FallenLeaves));
                    }

                    if rand::random::<f32>() < 0.01 {
                        iunit_service
                            .add_iunit(IUnit::new(IVec3::new(x, y, 1), IUnitKind::FallenBranch));
                    }

                    if rand::random::<f32>() < 0.04 {
                        iunit_service
                            .add_iunit(IUnit::new(IVec3::new(x, y, 1), IUnitKind::MixPebbles));
                    }

                    if rand::random::<f32>() < 0.02 {
                        unit_service.add_unit(Unit::new(
                            Uuid::new_v4(),
                            Vec3A::new(x as f32, y as f32, 0.5),
                            UnitKind::OakTree,
                        ));
                    }

                    if rand::random::<f32>() < 0.02 {
                        unit_service.add_unit(Unit::new(
                            Uuid::new_v4(),
                            Vec3A::new(x as f32, y as f32, 0.5),
                            UnitKind::BirchTree,
                        ));
                    }

                    if rand::random::<f32>() < 0.001 {
                        unit_service.add_unit(Unit::new(
                            Uuid::new_v4(),
                            Vec3A::new(x as f32, y as f32, 0.5),
                            UnitKind::DyingTree,
                        ));
                    }

                    if rand::random::<f32>() < 0.001 {
                        unit_service.add_unit(Unit::new(
                            Uuid::new_v4(),
                            Vec3A::new(x as f32, y as f32, 0.5),
                            UnitKind::FallenTree,
                        ));
                    }

                    if rand::random::<f32>() < 0.01 {
                        unit_service.add_unit(Unit::new(
                            Uuid::new_v4(),
                            Vec3A::new(x as f32, y as f32, 0.5),
                            UnitKind::MixRock,
                        ));
                    }

                    // generation rules end

                    self.init_flags.insert(pos);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate() {
        let mut iunit_service = IUnitService::default();
        let mut unit_service = UnitService::default();
        let mut gen_service = GenerationService::default();

        let aabb = IAabb3::new(IVec3::new(0, 0, 0), IVec3::new(8, 8, 8));
        gen_service.generate(aabb, &mut iunit_service, &mut unit_service);
    }
}
