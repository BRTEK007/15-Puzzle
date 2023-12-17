use macroquad::prelude::*;
use tween::Tweener;
use tween::QuadOut;
use macroquad::ui::{root_ui, hash, widgets};

struct Board {
    size: usize,
    slots: Vec<Vec<i32>>,
    empty_pos: (i32, i32),
    move_anim_data: MoveAnimData
}

struct MoveAnimData {
    slots: Vec<(i32, i32, i32)>,//slot x, slot y, slot value
    vec: (i32, i32),//move vector
    time: f32,//animation time from 0.0 to 1.0
    running: bool,
    tweener: Tweener<f32, f32, QuadOut> 
}

impl MoveAnimData {
    fn new(slots: Vec<(i32, i32, i32)>, vec: (i32, i32)) -> MoveAnimData {
        MoveAnimData {
            slots: slots, vec: vec, time: 0.0, running: false,
            tweener: Tweener::quad_out(0.0, 1.0, 0.15)
        }
    }
}

impl Board {
    fn new(n: usize) -> Board {
        let mut slots = vec![vec![0; n]; n];
        
        for y in 0..n {
            for x in 0..n {
                slots[y][x] = (x+y*n + 1) as i32;
            }
        }
    
        slots[n-1][n-1] = 0;

        Board{
          size: n,
          slots: slots,
          empty_pos: ((n-1) as i32, (n-1) as i32),
          move_anim_data: MoveAnimData::new(Vec::new(), (0,0))
        }
    }

    fn move_slot(&mut self, slot_pos: (i32, i32)){
        if self.move_anim_data.running {//animation has not finished
            return;
        }
        if slot_pos.0 == self.empty_pos.0 && slot_pos.1 == self.empty_pos.1 {//clicked on 0
            return;
        }
        
        let move_vec: (i32, i32);

        if slot_pos.0 == self.empty_pos.0{//same x, moving on y axis
            if slot_pos.1 < self.empty_pos.1 {//move positive y
                move_vec = (0, 1);
            } else {
                move_vec = (0, -1);
            }
        } else if slot_pos.1 == self.empty_pos.1{//same y, moving on x axis
            if slot_pos.0 < self.empty_pos.0 {//move positive x
                move_vec = (1, 0);
            } else {
                move_vec = (-1, 0);
            }
        } else {//cannot perform the move
            return
        }
          
        let mut moved_slots: Vec<(i32, i32, i32)> = Vec::new();
      
        let mut v = (slot_pos.0, slot_pos.1);
        while !(v.0 == self.empty_pos.0 && v.1 == self.empty_pos.1){
            moved_slots.push((v.0, v.1, self.slots[v.1 as usize][v.0 as usize]));
            v.0 += move_vec.0;
            v.1 += move_vec.1;
        }

        //set anim data slots that are gonna be moved
        self.move_anim_data = MoveAnimData::new(moved_slots.clone(), move_vec.clone());
        self.move_anim_data.running = true;

        //accually move the slots
        for slot in moved_slots.iter() {
            let new_slot = (slot.0 + move_vec.0, slot.1 + move_vec.1);
            self.slots[new_slot.1 as usize][new_slot.0 as usize] =  slot.2;
        }
        self.slots[slot_pos.1 as usize][slot_pos.0 as usize] = 0;
        self.empty_pos = slot_pos.clone();
    }

    fn update(&mut self, dt: f32) {
        if self.move_anim_data.running {
            self.move_anim_data.time = self.move_anim_data.tweener.move_by(dt);
            if self.move_anim_data.tweener.is_finished(){
                self.move_anim_data.running = false;
            }
        } 
    }

    fn swap_empty_with(&mut self, x: i32, y: i32) {
        let new_empty_pos = (self.empty_pos.0+x, self.empty_pos.1+y);
        self.slots[self.empty_pos.1 as usize][self.empty_pos.0 as usize] = self.slots[new_empty_pos.1 as usize][new_empty_pos.0 as usize];
        self.slots[new_empty_pos.1 as usize][new_empty_pos.0 as usize] = 0; 
        self.empty_pos = new_empty_pos;
    }

    fn mixup(&mut self) {
        let mut swaps = 0;
        let mut last_swap = 0;
        while swaps < (self.size*self.size*20) as i32{
            let r = rand::gen_range(0, 4);
            match r {
                0 => if self.empty_pos.1 > 0 && last_swap != 2{
                    self.swap_empty_with(0, -1);
                    swaps+=1;
                    last_swap=r;
                }  
                1 => if self.empty_pos.0 < (self.size-1) as i32 && last_swap != 3{
                    self.swap_empty_with(1, 0);
                    swaps+=1;
                    last_swap=r;
                }  
                2 => if self.empty_pos.1 < (self.size-1) as i32 && last_swap != 0{
                    self.swap_empty_with(0, 1);
                    swaps+=1;
                    last_swap=r;
                }  
                3 => if self.empty_pos.0 > 0 && last_swap != 1{
                    self.swap_empty_with(-1, 0);
                    swaps+=1;
                    last_swap=r;
                }
                _ => continue,
            }
        }
    }
}

struct BoardDimensions {
    board_size: f32,
    board_pos: (f32, f32),
    block_size: f32,
    block_offset: f32
}


fn get_board_dimensions(n: usize) -> BoardDimensions {
    let screen_height = screen_height();
    let screen_width = screen_width();
    let block_size = f32::min(0.66 * screen_width/(n as f32), 0.66 * screen_height/(n as f32));
    let block_offset = block_size*0.075;
    let board_size = (block_size+block_offset)*(n as f32)+block_offset;
    let left = (screen_width-board_size)*0.5;
    let top = (screen_height-board_size)*0.5;

    BoardDimensions{
        board_size: board_size,
        board_pos: (left, top),
        block_size: block_size,
        block_offset: block_offset
    }
}

fn get_block_color(val: i32, n: usize) -> Color{
    let block_colors = [
        Color::from_rgba(183, 18, 52, 255),
        Color::from_rgba(255, 88, 0, 255),
        Color::from_rgba(255, 213, 0, 255),
        Color::from_rgba(255, 255, 255, 255),
    ];
    let x = (val-1) % (n as i32);
    let y = (val-1) / (n as i32);
    let smallest_coord = i32::min(x, y);

    block_colors[smallest_coord as usize]
} 

fn draw_board(board: &Board,  board_dims: &BoardDimensions){
    let color_board = Color::from_rgba(25, 25, 25, 255);
    let color_font = Color::from_rgba(0, 0, 0, 255);

    
    draw_rectangle(board_dims.board_pos.0, board_dims.board_pos.1,
            board_dims.board_size, board_dims.board_size, color_board); 
        

        for y in 0..board.size {
            for x in 0..board.size {
                let mut pos_x = board_dims.board_pos.0 + board_dims.block_offset + (x as f32)*(board_dims.block_size+board_dims.block_offset);
                let mut pos_y = board_dims.board_pos.1 + board_dims.block_offset +(y as f32)*(board_dims.block_size+board_dims.block_offset);

                if board.move_anim_data.running {
                    if board.move_anim_data.slots.iter().any(|s| s.2 == board.slots[y][x]) {
                        let t = 1.0 - board.move_anim_data.time;
                        pos_x -= (board.move_anim_data.vec.0 as f32)*board_dims.block_size*t;
                        pos_y -= (board.move_anim_data.vec.1 as f32)*board_dims.block_size*t;
                    }
                }

                let text_str = board.slots[y][x].to_string();

                let text_mes: TextDimensions = measure_text(&text_str, None, (board_dims.block_size*0.75).round() as u16, 1.0);
                let text_pos_x = pos_x + (board_dims.block_size-text_mes.width)*0.5;
                let text_pos_y = pos_y + (board_dims.block_size-text_mes.height)*0.5 + text_mes.offset_y;
                
                if board.slots[y][x] != 0 {
                    draw_rectangle(pos_x, pos_y, board_dims.block_size, board_dims.block_size, get_block_color(board.slots[y][x], board.size)); 
                    draw_text(&text_str, text_pos_x, text_pos_y, board_dims.block_size*0.75, color_font);
                }
            }
        }
}


#[macroquad::main("fifteen_puzzle")]
async fn main() {
    rand::srand(macroquad::miniquad::date::now() as u64);

    let mut board = Board::new(4);
    //board.mixup();
    
    loop{
        let dt = get_frame_time();
        let board_dims: BoardDimensions = get_board_dimensions(board.size);
        
        if is_mouse_button_pressed(MouseButton::Left) {
            let press_pos = mouse_position();
           
            if ! (press_pos.0 < board_dims.board_pos.0 ||
                    press_pos.0 > board_dims.board_pos.0 + board_dims.board_size || 
                    press_pos.1 < board_dims.board_pos.1 ||
                    press_pos.1 > board_dims.board_pos.1 + board_dims.board_size) {
                let block_x = (((press_pos.0 - board_dims.board_pos.0)/board_dims.board_size) * (board.size as f32)) as i32;
                let block_y = (((press_pos.1 - board_dims.board_pos.1)/board_dims.board_size) * (board.size as f32)) as i32;
                
                board.move_slot((block_x, block_y));
                //println!("{} {}", block_x, block_y);
            }
        }

        board.update(dt);
   
        clear_background(Color::from_rgba(55, 55, 55, 255));

        draw_board(&board, &board_dims);

        widgets::Window::new(hash!(),
         vec2(50.0, 50.0),
         vec2(100.0, 150.0)).label("menu").titlebar(true)
        .ui(&mut *root_ui(), |ui| {
            if ui.button(None, "scramble") {
                board.mixup();
            }
            if ui.button(None, "reset") {
                board = Board::new(board.size);
            }
            if ui.button(None, "3x3") {
                board = Board::new(3);
            }
            if ui.button(None, "4x4"){
                board = Board::new(4);
            }
            if ui.button(None, "5x5"){
                board = Board::new(5);
            }
        });

        next_frame().await
    }
}
