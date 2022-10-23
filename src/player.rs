// use std::collections::HashMap;
use super::interaction::Item;
extern crate rand;
use rand::Rng;
use macroquad::prelude::*;
use super::map::CELL_SIZE;
use std::fmt;
use pathfinding::prelude::astar;
use macroquad::ui::{hash, root_ui, widgets, Skin,Style};
use super::map::AdvanceTileTypes;
use std::collections::HashMap;

pub fn character(char_type:&Character) -> [String; 4]{
    match char_type {
         Character::Priest => ["priest1_v2_1".to_string(),"priest1_v2_2".to_string(),"priest1_v2_3".to_string(),"priest1_v2_4".to_string()],
    }
}
#[derive(Default,Clone,Copy)]
pub struct Coordinates<T> {
    pub x: T,
    pub y: T
}
impl Coordinates<i16> {
    pub fn distance(&self, other: &Coordinates<i16>) -> f32{
        return (((self.x-other.x).pow(2)+(self.y-other.y).pow(2)) as f32).sqrt();
    }
}
pub struct Health {
    points: i16,
    base_health: i16
}
pub struct Inventory {
    pub storage: Vec<Item>,
    skin: Style
}
impl Default for Inventory {
    fn default() -> Self {
        Self {
            storage: Vec::new(),
            skin: root_ui()
            .style_builder()
            .background_margin(RectOffset::new(37.0, 37.0, 5.0, 5.0))
            .margin(RectOffset::new(10.0, 10.0, 0.0, 0.0)).build(),
        }
    }
}
impl Inventory {
    pub fn display_inventory(&self) {
        let button_skin = {let button_style = self.skin.clone();
            Skin {
                button_style,
                ..root_ui().default_skin()
            }};
        root_ui().push_skin(&button_skin);
            root_ui().window(hash!(), vec2(0., screen_height() / 15.), vec2(screen_width() / 6., screen_height() * 13.0 / 15.), |ui| {
                for each_item_index in 0..self.storage.len() {
                    if widgets::Button::new(&self.storage[each_item_index].item_type.to_string()[..])
                    .position(vec2(0.,0.+each_item_index as f32 * 25.))
                    .ui(ui) {
                        println!("Pushed", );
                    }
                    ui.separator();
                }
            });
    }

}
impl Health {
    pub fn adjust(&mut self, increment:i16) -> Option<bool>{
        self.points += increment;
        if self.points >= 0 {
            return Some(false);
        }
        None
    }
    pub fn new(base_health:i16) -> Self {
        Self {
            points: base_health,
            base_health: base_health,
        }
    }
}
pub struct Damage {
    ranged_damage: Option<i16>,
    cc_damage: i16,
    accuracy: f32,
}
impl Damage {
    pub fn deal(&self, distance: Option<i16>) -> i16 {
        if distance.is_none() {
            return self.cc_damage
        }
        let mut rng = rand::thread_rng();
        let chance: i16 = rng.gen_range(0..=100);
        let hit = (self.accuracy >= chance as f32) as i16;
        return self.ranged_damage.unwrap()*hit
    }
    pub fn new( base_cc:i16, base_range:i16, base_accuracy:f32) -> Self {
        Self {
            ranged_damage: Some(base_range),
            cc_damage: base_cc,
            accuracy: base_accuracy,
        }
    }
}
pub enum Character {
    Priest
}
impl fmt::Display for Character {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Character::Priest => write!(f,"Priest"),
        }
    }
}

pub struct PlayerCharacter {
    pub health: Health,
    pub damage: Damage,
    pub storage: Inventory,
    pub cor: Coordinates<i16>,
    pub frame: i16,
    pub character: Character
}
impl PlayerCharacter {
    pub fn intialise(base_health: i16, base_cc:i16, base_range:i16, base_accuracy:f32,spawn: Coordinates<i16>) -> Self{
        // let mut inventory_initial = [];
        // for row in 0..inventory_size.1 {
        //     for col in 0..inventory_size.0 {
        //          inventory_initial[0][1] =Item::Empty;
        //     }
        // }
        Self {
            health: Health::new(base_health),
            damage: Damage::new(base_cc,base_range,base_accuracy),
            storage: Inventory::default(),
            cor: spawn,
            frame: 0,
            character: Character::Priest,
        }
    }
    pub fn draw_player(&self, texture: Texture2D) {
        draw_texture_ex(texture,self.cor.x as f32 * CELL_SIZE, self.cor.y as f32 * CELL_SIZE, WHITE, DrawTextureParams {
            source: Some(Rect {
               x: 0.,y: 0.,w: CELL_SIZE ,h:CELL_SIZE
            }),
            ..Default::default()
        });
    }
    pub fn update_player_frame(&mut self) -> i16 {
        let frame = match self.frame {
         0 => 1,
         1 => 2,
         2 => 3,
         3 => 0,
         _ => 0,
         
        };
        self.frame = frame;
        return frame
    }
}

pub enum EntityType {
    Vampire
}
enum EntityStatus {
    Passive,
    Violent,
    Neutral
}
pub struct Entity {
    health: Health,
    damage: Damage,
    cor: Coordinates<i16>,
    entity_type: EntityType,
    entity_status: EntityStatus,
    movement_points: i16,
    frame: i16,

}
enum AIOptions {
    Move, Attack, Heal, LookAround, Panic, AvoidPlayer, Group
}
struct AIOption {
    aioid: i32,
    option: AIOptions,
}
impl PartialEq for AIOption {
    fn eq(&self, other: &Self) -> bool {
        self.aioid == other.aioid
    }
}
impl Eq for AIOption {}
impl Entity {
    pub fn intialise(base_health: i16, base_cc:i16, base_range:i16, base_accuracy:f32,spawn:Coordinates<i16>, e_type: EntityType,base_movement_points: i16) -> Self {
        let e_status = match &e_type {
            EntityType::Vampire => EntityStatus::Violent,
        };
        Self {
            health: Health::new(base_health),
            damage: Damage::new(base_cc,base_range,base_accuracy),
            cor: spawn,
            entity_type: e_type,
            entity_status: e_status,
            movement_points: base_movement_points,
            frame: 0,
        }
    }
    fn _attack(&self, target_pos: Coordinates<i16>) -> Option<i16> {
        if !matches!(self.entity_status,EntityStatus::Passive) {
            return Some(match (((self.cor.x-target_pos.x).pow(2)+(self.cor.y-target_pos.y).pow(2)) as f32).sqrt() >= (2.0 as f32).sqrt() {
                true => self.damage.cc_damage,
                false => self.damage.ranged_damage.unwrap()
            })
        }
        return None
    }
    fn _update_Entity_frame(&mut self) -> i16 {
        let frame = match self.frame {
         0 => 1,
         1 => 2,
         2 => 3,
         3 => 0,
         _ => 0,
         
        };
        self.frame = frame;
        return frame
    }
    fn _update_entity(self) {
        todo!()
    }
    pub fn consider_action(&mut self,tile_placement: &Vec<Vec<AdvanceTileTypes>>,player: Coordinates<i16>) -> Option<i16>{
        let moves = [(self.cor.x,self.cor.y+1),(self.cor.x,self.cor.y-1),(self.cor.x-1,self.cor.y),(self.cor.x+1,self.cor.y )];
        let possible_moves: Vec<(i16,i16)> = moves.into_iter().filter(|&x| true == match tile_placement[x.1 as usize][x.0 as usize] {
            AdvanceTileTypes::GenericFloor => true,
            AdvanceTileTypes::SmallCHest => true,
            AdvanceTileTypes::Bones => true,
            AdvanceTileTypes::Skull => true,
            AdvanceTileTypes::Chest => true,
            _ => false,
        }).collect();
        let mut rng = rand::thread_rng();
        if distance(player.x,player.y,self.cor.x,self.cor.y) <=  15 {
            let cost = possible_moves.iter().map(|x| distance(player.x,player.x,x.0,x.1)).collect::<Vec<i16>>();
            let mut proximity_cost = cost.iter();
            println!("cost = {:#?}, moves {:#?}", cost, possible_moves);
            if self.health.points <= (self.health.base_health/3){
                let max = proximity_cost.clone().max().unwrap();
                if *max >= 4 as i16 {
                    self.health.adjust(rng.gen_range(1..=10));
                } else {
                    let new_cor = possible_moves[proximity_cost.position(|&x| x == *max).unwrap()];
                    self.cor = Coordinates {x:new_cor.0,y:new_cor.1};
                }
            println!("picked {}", max);
            } else {
                if *proximity_cost.clone().min().unwrap() == 1 {
                    println!("deal", );
                    return Some(self.damage.deal(Some(1)));
                    
                }
                else {
                    let min = proximity_cost.clone().min().unwrap();
                    let new_cor = possible_moves[proximity_cost.position(|&x| x == *min).unwrap()];
                    self.cor = Coordinates {x:new_cor.0,y:new_cor.1};
                    
                    println!("picked min {}", min);
                }
            }
        }
        else {
            let random_move = rng.gen_range(0..possible_moves.len());
            self.cor = Coordinates {x:possible_moves[random_move].0,y:possible_moves[random_move].1};
        }
        None
    }
    pub fn draw_entity(&self) {
        // draw_texture_ex(*texture,self.cor.x as f32 * CELL_SIZE, self.cor.y as f32 * CELL_SIZE, WHITE, DrawTextureParams {
        //     source: Some(Rect {
        //        x: 0.,y: 0.,w: CELL_SIZE ,h:CELL_SIZE
        //     }),
        //     ..Default::default()
        draw_rectangle(self.cor.x as f32 * CELL_SIZE, self.cor.y as f32 * CELL_SIZE, CELL_SIZE, CELL_SIZE, RED)
        // });
}  

}
fn distance(x: i16,y:i16,x2:i16,y2:i16) -> i16 {
    (((x2-x).pow(2)+(y2-y).pow(2)) as f32).sqrt() as i16
}
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Pos(pub i32, pub i32);

impl Pos {
  fn distance(&self, other: &Pos) -> u32 {
    (self.0.abs_diff(other.0) + self.1.abs_diff(other.1)) as u32
  }

  fn successors(&self) -> Vec<(Pos, u32)> {
    let &Pos(x, y) = self;
    let next_tiles: Vec<(Pos, u32)> = vec![Pos(x+1,y+1), Pos(x+1,y-1), Pos(x-1,y+1), Pos(x-1,y-1),
         Pos(x+1,y+1), Pos(x+1,y-1), Pos(x-1,y+1), Pos(x-1,y-1)]
         .into_iter().map(|p| (p, 1)).collect();
  return next_tiles
}
}
pub fn output_shortest_path(goal: &Pos, start: &Pos) {
    let path = astar(start, |p| p.successors(), |p| p.distance(goal) / 3,
                   |p| p == goal);
    println!("{:#?}", path);
}
// struct EntityAI {
//     probability_of_actions: HashMap<AIOptions,f32>,
// }
// impl Default for EntityAI {
//     fn default() -> Self {
//         Self {

//         }
//     }
// }

// pub struct PathFinder {
//     goal: Coordinates<i16>,
//     start: Coordinates<i16>,
// }
// impl PathFinder {
//     fn verify_move(self,tile_placement: Vec<Vec<AdvanceTileTypes>>, verifying_move: Coordinates<i16>) -> bool{
//         match tile_placement[verifying_move.y as usize][verifying_move.x as usize] {
//         AdvanceTileTypes::BLCorner => false,
//         AdvanceTileTypes::BRCorner => false,
//         AdvanceTileTypes::TLCorner => false,
//         AdvanceTileTypes::TRCorner => false,
//         AdvanceTileTypes::LEdge => false,
//         AdvanceTileTypes::REdge => false,
//         AdvanceTileTypes::TEdge => false,
//         AdvanceTileTypes::BEdge => false,
//         AdvanceTileTypes::OBRCorner=> false,
//         AdvanceTileTypes::OBLCorner => false,
//         AdvanceTileTypes::OTLCorner => false,
//         AdvanceTileTypes::OTRCorner => false,
//         AdvanceTileTypes::Rock => false,
//         _ => true,
//         }
//     }
//     pub fn path_find