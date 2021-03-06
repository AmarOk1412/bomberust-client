/**
 * Copyright (c) 2019, Sébastien Blin <sebastien.blin@enconn.fr>
 * All rights reserved.
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions are met:
 *
 * * Redistributions of source code must retain the above copyright
 *  notice, this list of conditions and the following disclaimer.
 * * Redistributions in binary form must reproduce the above copyright
 *  notice, this list of conditions and the following disclaimer in the
 *  documentation and/or other materials provided with the distribution.
 * * Neither the name of the University of California, Berkeley nor the
 *  names of its contributors may be used to endorse or promote products
 *  derived from this software without specific prior written permission.
 *
 * THIS SOFTWARE IS PROVIDED BY THE REGENTS AND CONTRIBUTORS ``AS IS'' AND ANY
 * EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
 * WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
 * DISCLAIMED. IN NO EVENT SHALL THE REGENTS AND CONTRIBUTORS BE LIABLE FOR ANY
 * DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES
 * (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES;
 * LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND
 * ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
 * (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS
 * SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 **/

use super::super::utils::MapPlayer;
use super::{Walkable, Item};
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};
use std::any::Any;

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub enum Bonus {
    ImproveBombRadius,
    PunchBombs,
    ImproveSpeed,
    RepelBombs,
    MoreBombs,
    Custom(String)
}

impl Distribution<Bonus> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Bonus {
        match rng.gen_range(0, 4) {
            0 => Bonus::ImproveBombRadius,
            1 => Bonus::PunchBombs,
            2 => Bonus::ImproveSpeed,
            3 => Bonus::RepelBombs,
            _ => Bonus::MoreBombs,
        }
    }
}

impl Walkable for Bonus {
    fn walkable(&self, _p: &MapPlayer, _pos: &(usize, usize)) -> bool {
        true
    }

    fn explode_event(&self, _pos: &(usize, usize), _bomb_pos: &(usize, usize)) -> (bool, bool) {
        (true, true)
    }
}

#[typetag::serde]
impl Item for Bonus {
    fn name(&self) -> String {
        String::from("Bonus")
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn box_clone(&self) -> Box<dyn Item> {
        Box::new((*self).clone())
    }
}