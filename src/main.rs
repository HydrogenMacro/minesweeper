use std::cmp::{min, PartialEq};

use macroquad::prelude::*;
#[macroquad::main(conf)]
async fn main() {
	let mut current_game = Game::new(8, 24, 3, 3);
	let mut had_victory = false;
	let mut started = false;
	loop {
		clear_background(WHITE);

		if let Some((clicked_x, clicked_y)) = current_game.check_input() {
			if !started {
				current_game = Game::new(8, 24, clicked_x, clicked_y);
				started = true;
				had_victory = false;
			} else {
				if is_mouse_button_released(MouseButton::Right) {
					current_game.minefield.flag_tile(clicked_x, clicked_y);
				} else if let Err(_) = current_game.minefield.reveal_tile(clicked_x, clicked_y) {
					current_game.minefield.reveal_all_tiles();
					started = false;
				}
				if let Some(_) = current_game.minefield.check_victory() {
					println!("VICTORY");
					had_victory = true;
					started = false;
				}
			}
		}

		current_game.draw();
		if !started {
			if had_victory {
				draw_text("VICTORY!!!\nClick any tile to start new game", 0., screen_height() / 2., 48., SKYBLUE);
			} else {
				draw_text("Click any tile to start new game", 0., screen_height() / 2., 48., SKYBLUE);
			}
		}
		next_frame().await;
	}
}
fn conf() -> Conf {
	Conf {
		window_title: "Minesweeper".to_string(),
		high_dpi: true,
		..Default::default()
	}
}
#[derive(Debug, PartialEq, Eq, Clone)]
struct Game {
	minefield: Minefield,
}
impl Game {
	pub fn new(
		minefield_size: usize,
		num_mines: usize,
		initial_revelation_x: usize,
		initial_revelation_y: usize,
	) -> Game {
		let mut minefield = Minefield::new(minefield_size, num_mines);
		loop {
			match *minefield.get_tile_at_mut(initial_revelation_x, initial_revelation_y) {
				Tile::Uninitialized => unreachable!(),
				Tile::Mine(_) => {
					minefield = Minefield::new(minefield_size, num_mines);
				}
				Tile::Safe(ref mut tile_status, surrounding_mines) => {
					if surrounding_mines == 0 {
						*tile_status = TileStatus::Revealed;
						break;
					}
					minefield = Minefield::new(minefield_size, num_mines);
				}
			}
		}
		Game { minefield }
	}
	pub fn check_input(&mut self) -> Option<(usize, usize)> {
		if !(is_mouse_button_released(MouseButton::Left)
			|| is_mouse_button_released(MouseButton::Right))
		{
			return None;
		}
		let tile_size = screen_width().min(screen_height()) / self.minefield.size as f32;
		let minefield_origin = if screen_width() < screen_height() {
			vec2(0., (screen_height() - screen_width()) / 2.)
		} else {
			vec2((screen_width() - screen_height()) / 2., 0.)
		};
		let clicked_tile = ((Vec2::from(mouse_position()) - minefield_origin) / tile_size).floor();
		if clicked_tile.x < 0.
			|| clicked_tile.y < 0.
			|| clicked_tile.x as usize >= self.minefield.size
			|| clicked_tile.y as usize >= self.minefield.size
		{
			return None;
		}
		return Some((clicked_tile.x as usize, clicked_tile.y as usize));
	}
	pub fn draw(&self) {
		self.minefield.draw();
	}
}
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
enum TileStatus {
	Revealed,
	Flagged,
	Hidden,
}
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
enum Tile {
	Uninitialized,
	Mine(TileStatus),
	Safe(TileStatus, u8),
}
#[derive(Debug, PartialEq, Eq, Clone)]
struct Minefield {
	size: usize,
	remaining_mines: usize,
	tiles: Vec<Tile>,
}

impl Minefield {
	pub fn new(size: usize, num_mines: usize) -> Self {
		assert!(size * size >= num_mines);
		let mut tiles = vec![Tile::Uninitialized; size * size];
		let mut num_mines = num_mines;
		for i in 0..num_mines {
			let mine_index = rand::gen_range(0, size * size - 1);
			if tiles[mine_index] == Tile::Uninitialized {
				tiles[mine_index] = Tile::Mine(TileStatus::Hidden);
			} else {
				num_mines += 1;
			}
		}
		let mut minefield = Minefield {
			size,
			remaining_mines: num_mines,
			tiles,
		};
		for y in 0..size {
			for x in 0..size {
				if *minefield.get_tile_at(x, y) == Tile::Uninitialized {
					let surrounding_mines = minefield.get_tile_surrounding_mines(x, y);
					*minefield.get_tile_at_mut(x, y) =
						Tile::Safe(TileStatus::Hidden, surrounding_mines);
				}
			}
		}
		minefield
	}
	pub fn get_tile_surrounding_mines(&self, tile_x: usize, tile_y: usize) -> u8 {
		let (start_x, end_x) = if tile_x == 0 {
			(tile_x, tile_x + 1)
		} else if tile_x == self.size - 1 {
			(tile_x - 1, tile_x)
		} else {
			(tile_x - 1, tile_x + 1)
		};
		let (start_y, end_y) = if tile_y == 0 {
			(tile_y, tile_y + 1)
		} else if tile_y == self.size - 1 {
			(tile_y - 1, tile_y)
		} else {
			(tile_y - 1, tile_y + 1)
		};
		let mut surrounding_mines = 0;
		for surrounding_mine_y in start_y..=end_y {
			for surrounding_mine_x in start_x..=end_x {
				if surrounding_mine_x == tile_x && surrounding_mine_y == tile_y {
					continue;
				}
				if let Tile::Mine(_) = self.get_tile_at(surrounding_mine_x, surrounding_mine_y) {
					surrounding_mines += 1;
				}
			}
		}
		surrounding_mines
	}
	pub fn get_tile_at(&self, x: usize, y: usize) -> &Tile {
		&self.tiles[y * self.size + x]
	}
	pub fn get_tile_at_mut(&mut self, x: usize, y: usize) -> &mut Tile {
		&mut self.tiles[y * self.size + x]
	}
	pub fn flag_tile(&mut self, x: usize, y: usize) {
		match self.get_tile_at_mut(x, y) {
			Tile::Uninitialized => unreachable!(),
			Tile::Mine(tile_status) => {
				match tile_status {
					TileStatus::Revealed => {}
					TileStatus::Flagged => {
						*tile_status = TileStatus::Hidden;
					}
					TileStatus::Hidden => {
						*tile_status = TileStatus::Flagged;
					}
				}
			}
			Tile::Safe(tile_status, _) => {
				match tile_status {
					TileStatus::Revealed => {}
					TileStatus::Flagged => {
						*tile_status = TileStatus::Hidden;
					}
					TileStatus::Hidden => {
						*tile_status = TileStatus::Flagged;
					}
				}
			}
		}
	}
	pub fn reveal_tile(&mut self, x: usize, y: usize) -> Result<(), ()> {
		match self.get_tile_at_mut(x, y) {
			Tile::Uninitialized => unreachable!(),
			Tile::Mine(ref mut tile_status) => {
				*tile_status = TileStatus::Revealed;
				return Err(());
			}
			Tile::Safe(ref mut tile_status, surrounding_mines) => {
				*tile_status = TileStatus::Revealed;
			}
		}
		return Ok(());
	}
	pub fn reveal_all_tiles(&mut self) {
		for y in 0..self.size {
			for x in 0..self.size {
				match *self.get_tile_at_mut(x, y) {
					Tile::Uninitialized => unreachable!(),
					Tile::Mine(ref mut tile_status) => {
						*tile_status = TileStatus::Revealed;
					}
					Tile::Safe(ref mut tile_status, _) => {
						*tile_status = TileStatus::Revealed;
					}
				}
			}
		}
	}
	pub fn check_victory(&self) -> Option<()> {
		for y in 0..self.size {
			for x in 0..self.size {
				match self.get_tile_at(x, y) {
					Tile::Uninitialized => unreachable!(),
					Tile::Mine(tile_status) => {
						if let TileStatus::Hidden = tile_status {
							return None;
						}
						if let TileStatus::Revealed = tile_status {
							return None;
						}
					}
					Tile::Safe(tile_status, _) => {
						if let TileStatus::Flagged = tile_status {
							return None;
						}
						if let TileStatus::Hidden = tile_status {
							return None;
						}
					}
				}
			}
		}
		Some(())
	}
	pub fn draw(&self) {
		let tile_size = screen_width().min(screen_height()) / self.size as f32;
		let minefield_origin = if screen_width() < screen_height() {
			vec2(0., (screen_height() - screen_width()) / 2.)
		} else {
			vec2((screen_width() - screen_height()) / 2., 0.)
		};
		for y in 0..self.size {
			for x in 0..self.size {
				let pos_x = x as f32 * tile_size + minefield_origin.x;
				let pos_y = y as f32 * tile_size + minefield_origin.y;
				draw_rectangle(
					pos_x + (tile_size * 0.1),
					pos_y + (tile_size * 0.1),
					tile_size * 0.8,
					tile_size * 0.8,
					LIGHTGRAY,
				);
				if let Tile::Safe(tile_status, surrounding_mines) = self.get_tile_at(x, y) {
					match *tile_status {
						TileStatus::Revealed => {
							draw_text(
								&surrounding_mines.to_string(),
								pos_x + tile_size / 2.,
								pos_y + tile_size / 2.,
								tile_size * 0.8,
								BLACK,
							);
						}
						TileStatus::Flagged => {
							draw_rectangle(
								pos_x + (tile_size * 0.3),
								pos_y + (tile_size * 0.3),
								tile_size * 0.2,
								tile_size * 0.2,
								RED,
							);
						}
						TileStatus::Hidden => {}
					}
				}
				if let Tile::Mine(tile_status) = self.get_tile_at(x, y) {
					match tile_status {
						TileStatus::Revealed => {
							draw_rectangle(
								pos_x + (tile_size * 0.1),
								pos_y + (tile_size * 0.1),
								tile_size * 0.8,
								tile_size * 0.8,
								RED,
							);
						}
						TileStatus::Flagged => {
							draw_rectangle(
								pos_x + (tile_size * 0.3),
								pos_y + (tile_size * 0.3),
								tile_size * 0.2,
								tile_size * 0.2,
								RED,
							);
						}
						_ => {}
					}
				}
			}
		}
	}
}
