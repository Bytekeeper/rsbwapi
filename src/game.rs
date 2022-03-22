use crate::aimodule::AiModule;
use crate::bullet::Bullet;
use crate::cell::Wrap;
use crate::command::Commands;
use crate::force::Force;
use crate::player::Player;
use crate::predicate::IntoPredicate;
use crate::predicate::Predicate;
use crate::region::Region;
use crate::shm::Shm;
use crate::types::c_str_to_str;
use crate::unit::{Unit, UnitId, UnitInfo};
use crate::*;
use ahash::AHashMap;
use bwapi_wrapper::*;
use core::cell::{Ref, RefCell};
use delegate::delegate;
#[cfg(feature = "metrics")]
use metered::{hdr_histogram::HdrHistogram, measure, time_source::StdInstantMicros, ResponseTime};
use rstar::primitives::Rectangle;
use rstar::{Envelope, PointDistance, RTree, RTreeObject, AABB};
use std::ops::Deref;
use std::rc::Rc;

#[derive(Default, Debug, serde::Serialize)]
#[cfg(feature = "metrics")]
pub struct RsBwapiMetrics {
    frame_time: ResponseTime<RefCell<HdrHistogram>, StdInstantMicros>,
}

#[derive(Clone)]
pub struct Game {
    #[cfg(feature = "metrics")]
    metrics: std::rc::Rc<RsBwapiMetrics>,
    pub(crate) inner: Rc<RefCell<GameInternal>>,
}

pub struct UnitLocation {
    id: UnitId,
    location: Rectangle<[i32; 2]>,
}

impl RTreeObject for UnitLocation {
    type Envelope = AABB<[i32; 2]>;

    fn envelope(&self) -> Self::Envelope {
        self.location.envelope()
    }
}

impl PointDistance for UnitLocation {
    fn distance_2(&self, point: &[i32; 2]) -> i32 {
        self.location.distance_2(point)
    }
}

pub(crate) struct GameInternal {
    pub(crate) data: Shm<BWAPI_GameData>,
    units: Vec<Unit>,
    pub(crate) unit_infos: [Option<UnitInfo>; 10000],
    rtree: RTree<UnitLocation>,
    pub(crate) cmd: Commands,
    pub(crate) connected_units: AHashMap<usize, Vec<usize>>,
    pub(crate) loaded_units: AHashMap<usize, Vec<usize>>,
    pylons: Option<Vec<Unit>>,
    static_neutrals: Vec<Unit>,
    static_minerals: Vec<Unit>,
    static_geysers: Vec<Unit>,
    visible_units: Vec<Unit>,
}

impl PositionValidator for GameInternal {
    fn is_valid<const N: i32>(&self, pos: ScaledPosition<N>) -> bool {
        pos.x >= 0
            && pos.y >= 0
            && pos.x < self.map_width() * 32 / N
            && pos.y < self.map_height() * 32 / N
    }
}

impl PositionValidator for Game {
    fn is_valid<const N: i32>(&self, pos: ScaledPosition<N>) -> bool {
        self.inner.borrow().is_valid(pos)
    }
}

impl Wrap<&Game, &mut GameInternal> {
    fn match_start(&mut self) {
        self.inner.visible_units = (0..self.inner.data.initialUnitCount as usize)
            .filter(|&i| {
                self.inner.data.units[i].exists
                    && self.inner.data.units[i].type_ != UnitType::Unknown as i32
            })
            .map(|id| {
                Unit::new(id, self.outer.clone(), unsafe {
                    &*(&self.inner.data.units[id] as *const BWAPI_UnitData)
                })
            })
            .collect();

        for i in self.inner.visible_units.iter() {
            let id = i.get_id();
            self.inner.unit_infos[id] = Some(UnitInfo::new(&self.inner.data.units[id]));
            let ut = i.get_type();
            if ut == UnitType::Resource_Vespene_Geyser {
                self.inner.static_geysers.push(i.clone());
            }
            if ut.is_mineral_field() {
                self.inner.static_minerals.push(i.clone());
            }
            if i.get_player().is_neutral() {
                self.inner.static_neutrals.push(i.clone());
            }
        }
    }

    fn refresh(&mut self) {
        self.inner.units = self.inner.visible_units.clone();
        self.inner.loaded_units.clear();
        self.inner.connected_units.clear();
        self.inner.pylons = None;
        self.inner.rtree = RTree::bulk_load(
            self.inner
                .units
                .iter()
                .map(|u| UnitLocation {
                    id: u.get_id(),
                    location: Rectangle::from_corners(
                        [u.get_left(), u.get_top()],
                        [u.get_right(), u.get_bottom()],
                    ),
                })
                .collect(),
        );
    }
}

impl Wrap<&Game, &GameInternal> {
    pub fn is_replay(&self) -> bool {
        self.inner.data.isReplay
    }

    /// Returns a unit, if it exists or could exist still.
    /// For known dead units, it will return None.
    pub fn get_unit(&self, id: UnitId) -> Option<Unit> {
        self.inner.data.units[id].exists.then(|| {
            Unit::new(id, self.outer.clone(), unsafe {
                &*(&self.inner.data.units[id] as *const BWAPI_UnitData)
            })
        })
    }

    pub fn get_static_geysers(&self) -> Vec<Unit> {
        self.inner.static_geysers.clone()
    }

    pub fn get_static_minerals(&self) -> Vec<Unit> {
        self.inner.static_minerals.clone()
    }

    pub fn get_static_neutral_units(&self) -> Vec<Unit> {
        self.inner.static_neutrals.clone()
    }

    pub fn get_all_units(&self) -> Vec<Unit> {
        self.inner.units.clone()
    }

    fn get_selected_units(&self) -> Vec<Unit> {
        self.inner
            .data
            .selectedUnits
            .iter()
            .map(|id| {
                self.get_unit(*id as usize)
                    .expect("selected unit does not exist")
            })
            .collect()
    }

    fn get_closest_unit<P: Into<Position>, Pred: IntoPredicate<Unit>, R: Into<Option<u32>>>(
        &self,
        center: P,
        pred: Pred,
        radius: R,
    ) -> Option<Unit> {
        let center = center.into();
        let pred = pred.into_predicate();
        let radius = radius.into().unwrap_or(32000);
        let radius_squared = radius * radius;
        self.inner
            .rtree
            .nearest_neighbor_iter_with_distance_2(&[center.x, center.y])
            .take_while(|&(_, d_2)| d_2 <= radius_squared as i32)
            .map(|(ul, _)| self.get_unit(ul.id).expect("Unit from RTree to be present"))
            .find(|u| pred.test(u))
    }

    pub fn get_units_in_rectangle<A: Into<Position>, B: Into<Position>, P: IntoPredicate<Unit>>(
        &self,
        lt: A,
        rb: B,
        pred: P,
    ) -> Vec<Unit> {
        let lt = lt.into();
        let rb = rb.into();
        let pred = pred.into_predicate();
        self.inner
            .rtree
            .locate_in_envelope_intersecting(&AABB::from_corners([lt.x, lt.y], [rb.x, rb.y]))
            .map(|ul| self.get_unit(ul.id).expect("Unit from RTree to be present"))
            .filter(|u| pred.test(u))
            .collect()
    }

    fn get_player(&self, i: PlayerId) -> Option<Player> {
        if i >= self.inner.data.playerCount as usize {
            None
        } else {
            Some(Player::new(i, self.outer.clone()))
        }
    }

    fn get_closest_unit_in_rectangle<
        P: Into<Position>,
        Pred: IntoPredicate<Unit>,
        R: Into<crate::Rectangle<Position>>,
    >(
        &self,
        center: P,
        pred: Pred,
        rectangle: R,
    ) -> Option<Unit> {
        let center = center.into();
        let pred = pred.into_predicate();
        let rectangle = rectangle.into();
        let dx = (rectangle.tl.x - center.x)
            .abs()
            .max((rectangle.br.x - center.x).abs());
        let dy = (rectangle.tl.y - center.y)
            .abs()
            .max((rectangle.br.y - center.y).abs());
        let query_envelope = Rectangle::from_corners(
            [rectangle.tl.x, rectangle.tl.y],
            [rectangle.br.x, rectangle.br.y],
        )
        .envelope();
        let radius_2 = dx * dx + dy * dy;
        self.inner
            .rtree
            .nearest_neighbor_iter_with_distance_2(&[center.x, center.y])
            .take_while(|&(_, d_2)| d_2 <= radius_2)
            .filter(|(ul, _)| ul.location.envelope().intersects(&query_envelope))
            .map(|(ul, _)| self.get_unit(ul.id).expect("Unit from RTree to be present"))
            .find(|u| pred.test(u))
    }

    fn get_bullets(&self) -> Vec<Bullet> {
        (0..self.inner.data.bullets.len())
            .map(|id| Bullet::new(id, self.outer.clone()))
            .filter(|b| b.exists())
            .collect()
    }

    pub fn get_region_at<P: Into<Position>>(&self, p: P) -> Option<Region> {
        let Position { x, y } = p.into();

        let idx = self.inner.data.mapTileRegionId[x as usize / 32][y as usize / 32];
        let region_code = if idx & 0x2000 != 0 {
            let minitile_pos_x = (x & 0x1F) / 8;
            let minitile_pos_y = (y & 0x1F) / 8;
            let index = (idx & 0x1FFF) as usize;
            if index >= self.inner.data.mapSplitTilesMiniTileMask.len() {
                return None;
            }

            let mini_tile_mask = self.inner.data.mapSplitTilesMiniTileMask[index];
            if mini_tile_mask as usize >= self.inner.data.mapSplitTilesRegion1.len() {
                return None;
            }

            let minitile_shift = minitile_pos_x + minitile_pos_y * 4;
            if (mini_tile_mask >> minitile_shift) & 1 != 0 {
                self.inner.data.mapSplitTilesRegion2[index]
            } else {
                self.inner.data.mapSplitTilesRegion1[index]
            }
        } else {
            idx
        };
        self.get_region(region_code)
    }

    fn get_region(&self, id: u16) -> Option<Region> {
        if id >= self.inner.data.regionCount as u16 {
            None
        } else {
            Some(Region::new(id, self.outer.clone()))
        }
    }

    fn get_force(&self, force_id: i32) -> Force {
        if !(0..self.inner.data.forceCount).contains(&force_id) {
            panic!("Invalid force id {}", force_id);
        }
        let force_players = self
            .get_players()
            .iter()
            .filter(|p| p.force_id() == force_id)
            .cloned()
            .collect();
        Force::new(
            force_id as usize,
            &self.inner.data.forces[force_id as usize],
            force_players,
        )
    }

    fn get_forces(&self) -> Vec<Force> {
        (0..self.inner.data.forceCount)
            .map(|i| self.get_force(i))
            .collect()
    }

    fn neutral(&self) -> Player {
        self.get_player(self.inner.data.neutral as PlayerId)
            .expect("Neutral player to exist")
    }

    fn get_players(&self) -> Vec<Player> {
        (0..self.inner.data.playerCount as usize)
            .map(|i| Player::new(i, self.outer.clone()))
            .collect()
    }
}

impl Game {
    delegate! {
        to self.inner.deref().borrow() {
            pub fn get_frame_count(&self) -> i32;
            pub fn is_flag_enabled(&self, flag: Flag) -> bool;
            pub fn is_walkable<P: Into<WalkPosition>>(&self, wp: P) -> bool;
            pub fn is_in_game(&self) -> bool;
            pub fn is_buildable<P: Into<TilePosition>>(&self, tp: P) -> bool;
            pub fn is_explored<P: Into<TilePosition>>(&self, tp: P) -> bool;
            pub fn has_creep<P: Into<TilePosition>>(&self, tp: P) -> bool;
            pub fn is_visible<P: Into<TilePosition>>(&self, tp: P) -> bool;
            fn event_str(&self, i: usize) -> String;
        }
        to self.inner.deref().borrow_mut() {
            fn ensure_unit_info(&mut self, id: UnitId);
            fn unit_invisible(&mut self, id: UnitId);
        }
        to Wrap::new(self, &*self.inner.deref().borrow()) {
            pub fn is_replay(&self) -> bool;
            pub fn get_unit(&self, id: UnitId) -> Option<Unit>;
            pub fn get_static_geysers(&self) -> Vec<Unit>;
            pub fn get_static_minerals(&self) -> Vec<Unit>;
            pub fn get_static_neutral_units(&self) -> Vec<Unit>;
            pub fn get_all_units(&self) -> Vec<Unit>;
            pub fn get_selected_units(&self) -> Vec<Unit>;
            pub fn get_closest_unit<P: Into<Position>, Pred: IntoPredicate<Unit>, R: Into<Option<u32>>>(
                &self,
                center: P,
                pred: Pred,
                radius: R,
            ) -> Option<Unit>;
            pub fn get_units_in_rectangle<A: Into<Position>, B: Into<Position>, P: IntoPredicate<Unit>>(
                &self,
                lt: A,
                rb: B,
                pred: P,
            ) -> Vec<Unit>;
            pub fn get_player(&self, i: PlayerId) -> Option<Player>;
            pub fn get_closest_unit_in_rectangle<
                P: Into<Position>,
                Pred: IntoPredicate<Unit>,
                R: Into<crate::Rectangle<Position>>,
            >(
                &self,
                center: P,
                pred: Pred,
                rectangle: R,
            ) -> Option<Unit>;
            pub fn get_bullets(&self) -> Vec<Bullet>;
            pub fn get_region_at<P: Into<Position>>(&self, p: P) -> Option<Region>;
            pub(crate) fn get_region(&self, id: u16) -> Option<Region>;
            pub fn get_force(&self, force_id: i32) -> Force;
            pub fn get_forces(&self) -> Vec<Force>;
            pub fn neutral(&self) -> Player;
            pub fn get_players(&self) -> Vec<Player>;
        }

        to Wrap::new(self, &mut *self.inner.deref().borrow_mut()) {
            pub(crate) fn match_start(&self);
            fn refresh(&self);
        }
    }

    fn data(&self) -> Ref<'_, BWAPI_GameData> {
        Ref::map(self.inner.borrow(), |d| &*d.data)
    }

    pub(crate) fn handle_events(&mut self, module: &mut impl AiModule) {
        measure!(&self.metrics.clone().frame_time, {
            let event_count = self.data().eventCount;
            for i in 0..event_count {
                let event: BWAPIC_Event = self.inner.borrow().data.events[i as usize];
                use BWAPI_EventType_Enum::*;
                match event.type_ {
                    MatchStart => {
                        self.match_start();
                        self.refresh();
                        module.on_start(self);
                        // // No longer visible after the start event
                        self.inner.borrow_mut().visible_units.clear();
                    }
                    MatchFrame => {
                        self.refresh();
                        module.on_frame(self);
                    }
                    UnitCreate => {
                        let id = event.v1 as usize;
                        self.ensure_unit_info(id);
                        self.refresh();
                        module.on_unit_create(
                            self,
                            self.get_unit(id).expect("Created unit does not exist"),
                        );
                    }
                    UnitDestroy => {
                        let id = event.v1 as usize;
                        self.unit_invisible(id);
                        self.refresh();
                        module.on_unit_destroy(
                            self,
                            self.get_unit(id).expect("Destroyed unit already removed"),
                        );
                        self.inner.borrow_mut().unit_infos[id as usize] = Option::None;
                    }
                    UnitDiscover => {
                        let id = event.v1 as usize;
                        self.ensure_unit_info(id);
                        self.refresh();
                        module.on_unit_discover(
                            self,
                            self.get_unit(id).expect("Discovered unit does not exist"),
                        );
                    }
                    UnitEvade => {
                        let id = event.v1 as usize;
                        module.on_unit_evade(self, self.get_unit_ex(id));
                    }
                    UnitShow => {
                        let id = event.v1 as usize;
                        let unit = self.get_unit(id).expect("Shown unit does not exist");
                        self.inner.borrow_mut().visible_units.push(unit.clone());
                        self.refresh();
                        self.ensure_unit_info(id);
                        module.on_unit_show(self, unit);
                    }
                    UnitHide => {
                        let id = event.v1 as usize;
                        self.unit_invisible(id);
                        self.refresh();
                        module.on_unit_hide(
                            self,
                            self.get_unit(id).expect("Hidden unit does not exist"),
                        );
                    }
                    UnitMorph => {
                        let id = event.v1 as usize;
                        module.on_unit_morph(
                            self,
                            self.get_unit(id).expect("Morphing unit does not exist"),
                        );
                    }
                    UnitRenegade => {
                        let id = event.v1 as usize;
                        module.on_unit_morph(
                            self,
                            self.get_unit(id).expect("Renegading unit does not exist"),
                        );
                    }
                    UnitComplete => {
                        let id = event.v1 as usize;
                        self.ensure_unit_info(id);
                        self.refresh();
                        module.on_unit_complete(
                            self,
                            self.get_unit(id).expect("Completed unit does not exist"),
                        );
                    }
                    MatchEnd => {
                        let won = event.v1 != 0;
                        module.on_end(self, won);
                    }
                    MenuFrame => {}
                    SendText => module.on_send_text(self, self.event_str(event.v1 as usize)),
                    ReceiveText => {
                        module.on_receive_text(
                            self,
                            self.get_player(event.v1 as usize)
                                .expect("Text sending player does not exist"),
                            self.event_str(event.v2 as usize),
                        );
                    }
                    PlayerLeft => {
                        module.on_player_left(
                            self,
                            self.get_player(event.v1 as usize)
                                .expect("Leaving player does not exist"),
                        );
                    }
                    NukeDetect => {
                        module.on_nuke_detect(
                            self,
                            Position {
                                x: event.v1,
                                y: event.v2,
                            },
                        );
                    }
                    SaveGame => module.on_save_game(self, self.event_str(event.v1 as usize)),
                    None => {}
                }
            }
            let mut inner = self.inner.borrow_mut();
            std::mem::take(&mut inner.cmd).commit(&mut inner.data);
        })
    }
    pub fn get_geysers(&self) -> Vec<Unit> {
        self.get_all_units()
            .iter()
            .filter(|u| u.get_type() == UnitType::Resource_Vespene_Geyser)
            .cloned()
            .collect()
    }

    pub fn get_units_in_radius<P: Into<Position>, Pred: IntoPredicate<Unit>>(
        &self,
        position: P,
        radius: i32,
        pred: Pred,
    ) -> Vec<Unit> {
        let center = position.into();
        let radius_sq = radius * radius;
        let pred = pred.into_predicate();
        self.get_units_in_rectangle(
            (center.x - radius, center.y - radius),
            (center.x + radius, center.y + radius),
            |p: &Unit| {
                let d = center - p.get_position();
                d.x * d.x + d.y * d.y <= radius_sq && pred.test(p)
            },
        )
    }

    pub(crate) fn get_unit_ex(&self, id: UnitId) -> Unit {
        let data =
            unsafe { &*(&self.inner.deref().borrow().data.units[id] as *const BWAPI_UnitData) };
        Unit::new(id, self.clone(), data)
    }

    pub fn get_units_on_tile<TP: Into<TilePosition>, P: IntoPredicate<Unit>>(
        &self,
        tile: TP,
        pred: P,
    ) -> Vec<Unit> {
        let tile = tile.into();
        if !self.is_valid(tile) {
            vec![]
        } else {
            let p = tile.to_position();
            self.get_units_in_rectangle((p.x, p.y), (p.x + 32, p.y + 32), pred)
        }
    }

    pub fn can_build_here<'a, P: Into<TilePosition>, B: Into<Option<&'a Unit>>>(
        &'a self,
        builder: B,
        position: P,
        type_: UnitType,
        check_explored: bool,
    ) -> BwResult<bool> {
        let builder = builder.into();
        let position = if builder.is_some() && type_.is_addon() {
            position.into() + TilePosition { x: 4, y: 1 }
        } else {
            position.into()
        };

        let lt = position;
        let rb = lt + type_.tile_size();

        if !self.is_valid(lt) || !self.is_valid(rb.to_position() - Position { x: 1, y: 1 }) {
            return Err(Error::Unbuildable_Location);
        }

        if type_.is_refinery() {
            return Ok(self
                .get_geysers()
                .iter()
                .find(|x| x.get_tile_position() == position)
                .map_or(false, |x| {
                    !(x.is_visible() && x.get_type() != UnitType::Resource_Vespene_Geyser)
                }));
        }

        for x in lt.x..rb.x {
            for y in lt.y..rb.y {
                if !self.is_buildable((x, y)) || (check_explored && !self.is_explored((x, y))) {
                    return Ok(false);
                }
            }
        }

        if let Some(builder) = builder {
            if !builder.get_type().is_building() {
                if !builder.has_path(lt.to_position() + type_.tile_size().to_position() / 2) {
                    return Err(Error::Unreachable_Location);
                }
            } else if !builder.get_type().is_flying_building()
                && type_ != UnitType::Zerg_Nydus_Canal
                && !type_.is_flag_beacon()
            {
                return Ok(false);
            }
        }

        if type_ != UnitType::Special_Start_Location {
            let targ_pos = lt.to_position() + type_.tile_size().to_position() / 2;
            let collides_with_units = self
                .get_units_in_rectangle(
                    lt.to_position(),
                    rb.to_position(),
                    !Unit::is_flying.into_predicate()
                        & !Unit::is_loaded.into_predicate()
                        & |u: &Unit| {
                            builder.map_or(true, |b| b != u)
                                && u.get_left() <= targ_pos.x + type_.dimension_right()
                                && u.get_top() <= targ_pos.y + type_.dimension_down()
                                && u.get_right() >= targ_pos.x - type_.dimension_left()
                                && u.get_bottom() >= targ_pos.y - type_.dimension_up()
                        },
                )
                .iter()
                .any(|u| !(u.get_type().is_addon() && u.get_type().can_move()));
            if collides_with_units {
                return Ok(false);
            }

            let needs_creep = type_.requires_creep();
            if type_.get_race() != Race::Zerg || needs_creep {
                for x in lt.x..rb.x {
                    for y in lt.y..rb.y {
                        if needs_creep != self.has_creep((x, y)) {
                            return Ok(false);
                        }
                    }
                }
            }

            if type_.is_resource_depot() {
                for m in self.get_static_minerals() {
                    let tp = m.get_initial_tile_position();
                    if self.is_visible(tp) || self.is_visible((tp.x + 1, tp.y)) && !m.exists() {
                        continue;
                    }
                    if tp.x > lt.x - 5 && tp.y > lt.y - 4 && tp.x < lt.x + 7 && tp.y < lt.y + 6 {
                        return Ok(false);
                    }
                }
                for g in self.get_static_geysers() {
                    let tp = g.get_initial_tile_position();
                    if tp.x > lt.x - 7 && tp.y > lt.y - 5 && tp.x < lt.x + 7 && tp.y < lt.y + 6 {
                        return Ok(false);
                    }
                }
            }
        }

        if let Some(builder) = builder {
            if builder.get_type().is_addon()
                && type_.is_addon()
                && !self.can_build_here(
                    builder,
                    lt - TilePosition { x: 4, y: 1 },
                    builder.get_type(),
                    check_explored,
                )?
            {
                return Ok(false);
            }
        }
        Ok(true)
    }

    pub fn has_path<S: Into<Position>, D: Into<Position>>(
        &self,
        source: S,
        destination: D,
    ) -> bool {
        let source = source.into();
        let destination = destination.into();
        if self.is_valid(source) && self.is_valid(destination) {
            let rgn_a = self.get_region_at(source);
            let rgn_b = self.get_region_at(destination);
            if let (Some(rgn_a), Some(rgn_b)) = (rgn_a, rgn_b) {
                return rgn_a.get_region_group_id() == rgn_b.get_region_group_id();
            }
        }
        false
    }

    pub fn observers(&self) -> Vec<Player> {
        self.get_players()
            .iter()
            .filter(|p| p.is_observer())
            .cloned()
            .collect()
    }

    pub fn enemies(&self) -> Vec<Player> {
        let self_ = self.self_();
        if let Some(self_) = self_ {
            self.get_players()
                .iter()
                .filter(|p| p.is_enemy(&self_))
                .cloned()
                .collect()
        } else {
            vec![]
        }
    }

    pub fn can_command(&self, this_unit: &Unit) -> Result<bool, Error> {
        if Some(this_unit.get_player()) != self.self_() {
            return Err(Error::Unit_Not_Owned);
        }

        if !this_unit.exists() {
            return Err(Error::Unit_Does_Not_Exist);
        }

        if this_unit.is_locked_down()
            || this_unit.is_maelstrommed()
            || this_unit.is_stasised()
            || !this_unit.is_powered()
            || this_unit.get_order() == Order::ZergBirth
            || this_unit.is_loaded()
        {
            if !this_unit.get_type().produces_larva() {
                return Err(Error::Unit_Busy);
            } else {
                for larva in this_unit.get_larva() {
                    if self.can_command(&larva).unwrap_or(false) {
                        return Ok(true);
                    }
                }
                return Err(Error::Unit_Busy);
            }
        }

        let u_type = this_unit.get_type();
        if u_type == UnitType::Protoss_Interceptor
            || u_type == UnitType::Terran_Vulture_Spider_Mine
            || u_type == UnitType::Spell_Scanner_Sweep
            || u_type == UnitType::Special_Map_Revealer
        {
            return Err(Error::Incompatible_UnitType);
        }

        if this_unit.is_completed()
            && (u_type == UnitType::Protoss_Pylon
                || u_type == UnitType::Terran_Supply_Depot
                || u_type.is_resource_container()
                || u_type == UnitType::Protoss_Shield_Battery
                || u_type.is_powerup()
                || (u_type.is_special_building() && !u_type.is_flag_beacon()))
        {
            return Err(Error::Incompatible_State);
        }

        if !this_unit.is_completed() && !u_type.is_building() && !this_unit.is_morphing() {
            return Err(Error::Incompatible_State);
        }
        Ok(true)
    }

    pub fn can_make<'a, B: Into<Option<&'a Unit>>>(
        &'a self,
        builder: B,
        type_: UnitType,
    ) -> BwResult<bool> {
        if let Some(self_) = self.self_() {
            if !self_.is_unit_available(type_) {
                return Err(Error::Access_Denied);
            }
            let builder = builder.into();
            let required_type = type_.what_builds().0;
            if let Some(builder) = builder {
                if builder.get_player() != self_ {
                    return Err(Error::Unit_Not_Owned);
                }
                let builder_type = builder.get_type();
                if type_ == UnitType::Zerg_Nydus_Canal && builder_type == UnitType::Zerg_Nydus_Canal
                {
                    if !builder.is_completed() {
                        return Err(Error::Unit_Busy);
                    }
                    if builder.get_nydus_exit().is_some() {
                        return Err(Error::Unknown);
                    }
                    return Ok(true);
                }

                if required_type == UnitType::Zerg_Larva && builder_type.produces_larva() {
                    if builder.get_larva().is_empty() {
                        return Err(Error::Unit_Does_Not_Exist);
                    }
                } else if builder_type != required_type {
                    return Err(Error::Incompatible_UnitType);
                }

                let mut max_amt: i32;
                match builder_type {
                    UnitType::Protoss_Carrier | UnitType::Hero_Gantrithor => {
                        max_amt = 4;
                        if self_.get_upgrade_level(UpgradeType::Carrier_Capacity) > 0
                            || builder_type == UnitType::Hero_Gantrithor
                        {
                            max_amt += 4;
                        }

                        if builder.get_interceptor_count()
                            + builder.get_training_queue().len() as i32
                            >= max_amt
                        {
                            return Err(Error::Insufficient_Space);
                        }
                    }
                    UnitType::Protoss_Reaver | UnitType::Hero_Warbringer => {
                        max_amt = 5;
                        if self_.get_upgrade_level(UpgradeType::Reaver_Capacity) > 0
                            || builder_type == UnitType::Hero_Warbringer
                        {
                            max_amt += 5;
                        }

                        if builder.get_scarab_count() + builder.get_training_queue().len() as i32
                            >= max_amt
                        {
                            return Err(Error::Insufficient_Space);
                        }
                    }
                    _ => (),
                }
            }

            if self_.minerals() < type_.mineral_price() {
                return Err(Error::Insufficient_Minerals);
            }

            if self_.gas() < type_.gas_price() {
                return Err(Error::Insufficient_Gas);
            }

            let type_race = type_.get_race();
            let supply_required = type_.supply_required()
                * (if type_.is_two_units_in_one_egg() {
                    2
                } else {
                    1
                });
            if supply_required > 0
                && self_.supply_total_for(type_race)
                    < self_.supply_used_by(type_race) + supply_required
                        - (if required_type.get_race() == type_race {
                            required_type.supply_required()
                        } else {
                            0
                        })
            {
                return Err(Error::Insufficient_Supply);
            }

            let mut addon = UnitType::None;
            for it in type_.required_units() {
                if it.0.is_addon() {
                    addon = it.0;
                }

                if !self_.has_unit_type_requirement(it.0, it.1) {
                    return Err(Error::Insufficient_Tech);
                }
            }

            if type_.required_tech() != TechType::None
                && !self_.has_researched(type_.required_tech())
            {
                return Err(Error::Insufficient_Tech);
            }

            if let Some(builder) = builder {
                if addon != UnitType::None
                    && addon.what_builds().0 == type_.what_builds().0
                    && (builder.get_addon().is_none()
                        || builder.get_addon().map(|a| a.get_type()) == Some(addon))
                {
                    return Err(Error::Insufficient_Tech);
                }
            }

            Ok(true)
        } else {
            Err(Error::Unit_Not_Owned)
        }
    }

    pub fn allies(&self) -> Vec<Player> {
        let self_ = self.self_();
        if let Some(self_) = self_ {
            self.get_players()
                .iter()
                .filter(|p| p.is_ally(&self_))
                .cloned()
                .collect()
        } else {
            vec![]
        }
    }

    pub fn enemy(&self) -> Option<Player> {
        self.get_player(self.data().enemy as PlayerId)
    }

    pub fn self_(&self) -> Option<Player> {
        self.get_player(self.data().self_ as PlayerId)
    }

    #[cfg(feature = "metrics")]
    pub fn get_metrics(&self) -> &RsBwapiMetrics {
        &self.metrics
    }
}

impl GameInternal {
    pub fn is_in_game(&self) -> bool {
        self.data.isInGame
    }

    fn unit_invisible(&mut self, id: UnitId) {
        let index = self
            .visible_units
            .iter()
            .position(|u| u.get_id() as usize == id);
        if let Some(index) = index {
            self.visible_units.swap_remove(index);
        }
    }

    pub fn countdown_timer(&self) -> i32 {
        self.data.countdownTimer
    }

    pub fn elapsed_time(&self) -> i32 {
        self.data.elapsedTime
    }

    pub fn get_average_fps(&self) -> f64 {
        self.data.averageFPS
    }

    pub fn has_power<TP: Into<TilePosition>, TS: Into<TilePosition>>(
        &mut self,
        position: TP,
        size: TS,
    ) -> bool {
        let position = position.into().to_position() + size.into().to_position() * 16;
        self.has_power_precise(position)
    }

    pub fn has_power_precise<P: Into<Position>>(&mut self, position: P) -> bool {
        static B_PSI_FIELD_MASK: [[u8; 16]; 10] = [
            [0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0],
            [0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0],
            [0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0],
            [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
            [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
            [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
            [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
            [0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0],
            [0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0],
            [0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0],
        ];

        let pylons = self.pylons.get_or_insert_with(|| {
            self.units
                .iter()
                .filter(|u| u.get_type() == UnitType::Protoss_Pylon)
                .cloned()
                .collect()
        });
        let Position { x, y } = position.into();

        for i in pylons {
            if !i.exists() || !i.is_completed() {
                continue;
            }

            let p = i.get_position();
            if (p.x - x).abs() >= 256 {
                continue;
            }

            if (p.y - y).abs() >= 160 {
                continue;
            }

            if B_PSI_FIELD_MASK[(y - p.y + 160) as usize / 32][(x - p.x + 256) as usize / 32] != 0 {
                return true;
            }
        }
        false
    }

    pub fn is_battle_net(&self) -> bool {
        self.data.isBattleNet
    }

    pub fn is_debug(&self) -> bool {
        self.data.isDebug
    }

    pub fn is_multiplayer(&self) -> bool {
        self.data.isMultiplayer
    }

    pub fn is_walkable<P: Into<WalkPosition>>(&self, wp: P) -> bool {
        let p = wp.into();
        self.data.isWalkable[p.x as usize][p.y as usize]
    }

    pub fn is_visible<P: Into<TilePosition>>(&self, tp: P) -> bool {
        let p = tp.into();
        self.data.isVisible[p.x as usize][p.y as usize]
    }

    pub fn is_buildable<P: Into<TilePosition>>(&self, tp: P) -> bool {
        let p = tp.into();
        self.data.isBuildable[p.x as usize][p.y as usize]
    }

    pub fn is_explored<P: Into<TilePosition>>(&self, tp: P) -> bool {
        let p = tp.into();
        self.data.isExplored[p.x as usize][p.y as usize]
    }

    pub fn is_gui_enabled(&self) -> bool {
        self.data.hasGUI
    }

    pub fn is_flag_enabled(&self, flag: Flag) -> bool {
        *self.data.flags.get(flag as usize).unwrap_or(&false)
    }

    pub fn is_lat_com_enabled(&self) -> bool {
        self.data.hasLatCom
    }

    pub fn is_paused(&self) -> bool {
        self.data.isPaused
    }

    pub fn is_replay(&self) -> bool {
        self.data.isReplay
    }

    pub fn has_creep<P: Into<TilePosition>>(&self, tp: P) -> bool {
        let p = tp.into();
        self.data.hasCreep[p.x as usize][p.y as usize]
    }

    pub fn get_ground_height<P: Into<TilePosition>>(&self, tp: P) -> i32 {
        let p = tp.into();
        self.data.getGroundHeight[p.x as usize][p.y as usize]
    }

    pub fn get_instance_number(&self) -> i32 {
        self.data.instanceID
    }

    pub fn get_key_state(&self, key: Key) -> bool {
        self.data.keyState[key as usize]
    }

    pub fn get_latency(&self) -> i32 {
        self.data.latency
    }

    pub fn get_latency_frames(&self) -> i32 {
        self.data.latencyFrames
    }

    pub fn get_latency_time(&self) -> i32 {
        self.data.latencyTime
    }

    pub fn get_mouse_position(&self) -> Position {
        Position {
            x: self.data.mouseX,
            y: self.data.mouseY,
        }
    }

    pub fn get_mouse_state(&self, button: MouseButton) -> bool {
        self.data.mouseState[button as usize]
    }

    pub fn map_height(&self) -> i32 {
        self.data.mapHeight
    }

    pub fn map_width(&self) -> i32 {
        self.data.mapWidth
    }

    pub fn map_hash(&self) -> String {
        c_str_to_str(&self.data.mapHash)
    }

    pub fn map_file_name(&self) -> String {
        c_str_to_str(&self.data.mapFileName)
    }

    pub fn map_path_name(&self) -> String {
        c_str_to_str(&self.data.mapPathName)
    }

    pub fn map_name(&self) -> String {
        c_str_to_str(&self.data.mapName)
    }

    pub fn get_client_version(&self) -> i32 {
        self.data.client_version
    }

    pub fn get_damage_from<P1: Into<Option<Player>>, P2: Into<Option<Player>>>(
        &self,
        from_type: UnitType,
        to_type: UnitType,
        from_player: P1,
        to_player: P2,
    ) -> i32 {
        static DAMAGE_RATIO: [[i32; UnitSizeType::MAX as usize]; DamageType::MAX as usize] = [
            // Ind, Sml, Med, Lrg, Non, Unk
            [0, 0, 0, 0, 0, 0],       // Independent
            [0, 128, 192, 256, 0, 0], // Explosive
            [0, 256, 128, 64, 0, 0],  // Concussive
            [0, 256, 256, 256, 0, 0], // Normal
            [0, 256, 256, 256, 0, 0], // Ignore_Armor
            [0, 0, 0, 0, 0, 0],       // None
            [0, 0, 0, 0, 0, 0],       // Unknown
        ];
        let wpn = if to_type.is_flyer() {
            from_type.air_weapon()
        } else {
            from_type.ground_weapon()
        };
        if wpn == WeaponType::None || wpn == WeaponType::Unknown {
            return 0;
        }
        let mut dmg = if let Some(from_player) = from_player.into() {
            from_player.damage(wpn)
        } else {
            wpn.damage_amount() * wpn.damage_factor()
        };

        if wpn.damage_type() != DamageType::Ignore_Armor {
            if let Some(to_player) = to_player.into() {
                dmg -= dmg.min(to_player.armor(to_type));
            }
        }
        dmg * DAMAGE_RATIO[wpn.damage_type() as usize][to_type.size() as usize] / 256
    }

    pub fn get_damage_to<P1: Into<Option<Player>>, P2: Into<Option<Player>>>(
        &self,
        to_type: UnitType,
        from_type: UnitType,
        to_player: P2,
        from_player: P1,
    ) -> i32 {
        self.get_damage_from(from_type, to_type, from_player, to_player)
    }

    pub fn get_events(&self) -> Vec<BWAPIC_Event> {
        (0..self.data.eventCount as usize)
            .map(|i| self.data.events[i])
            .collect()
    }

    pub fn get_fps(&self) -> i32 {
        self.data.fps
    }

    pub fn get_frame_count(&self) -> i32 {
        self.data.frameCount
    }

    pub fn get_nuke_dots(&self) -> Vec<Position> {
        (0..self.data.nukeDotCount as usize)
            .map(|i| self.data.nukeDots[i])
            .map(|p| Position { x: p.x, y: p.y })
            .collect()
    }

    pub fn get_random_seed(&self) -> u32 {
        self.data.randomSeed
    }

    pub fn get_remaining_latency_frames(&self) -> i32 {
        self.data.remainingLatencyFrames
    }

    pub fn get_remaining_latency_time(&self) -> i32 {
        self.data.remainingLatencyTime
    }

    pub fn get_replay_frame_count(&self) -> i32 {
        self.data.replayFrameCount
    }

    pub fn get_revision(&self) -> i32 {
        self.data.revision
    }

    pub fn get_screen_position(&self) -> Position {
        Position {
            x: self.data.screenX,
            y: self.data.screenY,
        }
    }

    pub fn get_start_locations(&self) -> Vec<TilePosition> {
        (0..self.data.startLocationCount as usize)
            .map(|i| self.data.startLocations[i])
            .map(|p| TilePosition { x: p.x, y: p.y })
            .collect()
    }

    fn event_str(&self, i: usize) -> String {
        c_str_to_str(&self.data.eventStrings[i])
    }
    fn ensure_unit_info(&mut self, id: UnitId) {
        self.unit_infos[id] = Some(UnitInfo::new(&self.data.units[id]));
    }
}

impl Game {
    pub(crate) fn new(data: Shm<BWAPI_GameData>) -> Self {
        Game {
            #[cfg(feature = "metrics")]
            metrics: Default::default(),
            inner: Rc::new(RefCell::new(GameInternal {
                data,
                connected_units: AHashMap::new(),
                loaded_units: AHashMap::new(),
                pylons: None,
                rtree: RTree::new(),
                cmd: Commands::new(),
                static_geysers: vec![],
                static_minerals: vec![],
                static_neutrals: vec![],
                unit_infos: [None; 10000],
                visible_units: vec![],
                units: vec![],
            })),
        }
    }
}
