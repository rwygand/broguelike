use bracket_lib::prelude::*;
use specs::prelude::*;
use crate::map::{Map, Position};
use crate::player::player_input;
use crate::{render::Renderable, render, State};

#[derive(PartialEq, Copy, Clone)]
pub enum RunState { Paused, Running }

impl GameState for State {
    fn tick(&mut self, ctx : &mut BTerm) {
        ctx.cls();

        if self.runstate == RunState::Running {
            self.run_systems();
            self.runstate = RunState::Paused;
        } else {
            self.runstate = player_input(self, ctx);
        }

        render::draw_map(&self.ecs, ctx);

        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();
        let map = self.ecs.fetch::<Map>();

        for (pos, render) in (&positions, &renderables).join() {
            let idx = map.xy_idx(pos.x, pos.y);
            if map.visible_tiles[idx] { ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph) }
        }
    }
}