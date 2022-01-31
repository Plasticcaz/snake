use macroquad::{prelude::*, ui::root_ui};

const PART_WIDTH: f32 = 10.0;
const PART_HEIGHT: f32 = 10.0;

#[derive(Clone, Copy, Debug)]
enum Direction {
    North,
    South,
    West,
    East,
}

fn is_opposite_of(one: Direction, other: Direction) -> bool {
    use Direction::*;
    matches!(
        (one, other),
        (East, West) | (West, East) | (North, South) | (South, North)
    )
}

#[derive(Clone, Copy)]
struct Position((f32, f32));

fn are_basically_eq(this: Position, other: Position) -> bool {
    let Position((x, y)) = this;
    let Position((o_x, o_y)) = other;

    let dx = x - o_x;
    let dy = y - o_y;

    dx > -0.1 && dx < 0.1 && dy > -0.1 && dy < 0.1
}

fn next_position(from: Position, direction: Direction) -> Position {
    let Position((old_x, old_y)) = from;

    let (dx, dy) = match direction {
        Direction::North => (old_x, old_y - 1.0),
        Direction::South => (old_x, old_y + 1.0),
        Direction::West => (old_x - 1.0, old_y),
        Direction::East => (old_x + 1.0, old_y),
    };

    Position((dx, dy))
}

struct PlayState {
    walls: Vec<Position>,

    parts: Vec<Position>,
    /// The current direction we are headed.
    direction: Direction,
    /// The direction we will switch to at next movement.
    next_direction: Direction,

    fruit: Position,

    time_since_last_move: f32,
    dead: bool,
}

fn random_position_on_board() -> Position {
    let x = rand::gen_range(1, 9) as f32;
    let y = rand::gen_range(1, 9) as f32;

    Position((x, y))
}

fn reset_state() -> PlayState {
    let walls = {
        let mut walls = Vec::new();

        for x in 0..11 {
            walls.push(Position((x as f32, 0.0)));
            walls.push(Position((x as f32, 10.0)));
        }
        for y in 1..10 {
            walls.push(Position((0.0, y as f32)));
            walls.push(Position((10.0, y as f32)));
        }

        walls
    };

    PlayState {
        walls,
        parts: vec![Position((2.0, 1.0)), Position((1.0, 1.0))],
        direction: Direction::East,
        next_direction: Direction::East,
        time_since_last_move: 0.0,
        fruit: random_position_on_board(),
        dead: false,
    }
}

fn extend_snake_body(state: &mut PlayState) {
    let Position((x, y)) = match state.parts.last().cloned() {
        Some(position) => position,
        None => return,
    };

    let new_part = match state.direction {
        Direction::North => Position((x, y + 1.0)),
        Direction::South => Position((x, y - 1.0)),
        Direction::West => Position((x + 1.0, y)),
        Direction::East => Position((x - 1.0, y)),
    };

    state.parts.push(new_part);
}

fn update(state: &mut PlayState) {
    if is_key_pressed(KeyCode::Escape) {
        *state = reset_state();
    }

    if state.dead {
        return;
    }
    let dt = get_frame_time();

    state.next_direction = input_to_direction(state.direction, KeyCode::Left, Direction::West)
        .or_else(|| input_to_direction(state.direction, KeyCode::Right, Direction::East))
        .or_else(|| input_to_direction(state.direction, KeyCode::Up, Direction::North))
        .or_else(|| input_to_direction(state.direction, KeyCode::Down, Direction::South))
        .unwrap_or(state.next_direction);

    state.time_since_last_move += dt;
    if state.time_since_last_move < 0.2 {
        return;
    }

    state.time_since_last_move = 0.0;
    state.direction = state.next_direction;
    let mut next_position = next_position(state.parts[0], state.direction);
    for part in state.parts.iter_mut() {
        std::mem::swap(part, &mut next_position);
    }

    // Collision check
    let head = state.parts[0];
    // Check for collisions with body:
    if state
        .parts
        .iter()
        .skip(1)
        .any(|it| are_basically_eq(*it, head))
    {
        state.dead = true;
    }

    // Check for collisions with walls:
    if state.walls.iter().any(|it| are_basically_eq(*it, head)) {
        state.dead = true;
    }

    if are_basically_eq(head, state.fruit) {
        extend_snake_body(state);
        state.fruit = random_position_on_board();
    }
}

fn render(state: &PlayState) {
    clear_background(GRAY);

    fn draw_block(x: f32, y: f32, color: Color) {
        draw_rectangle(
            x * PART_WIDTH,
            y * PART_HEIGHT,
            PART_WIDTH,
            PART_HEIGHT,
            color,
        );

        draw_rectangle_lines(
            x * PART_WIDTH,
            y * PART_HEIGHT,
            PART_WIDTH,
            PART_HEIGHT,
            0.1,
            DARKBROWN,
        );
    }

    for Position((x, y)) in state.walls.iter().cloned() {
        draw_block(x, y, BLACK)
    }

    let mut rendered_head = false;
    for Position((x, y)) in state.parts.iter().cloned() {
        let color = if rendered_head {
            RED
        } else {
            rendered_head = true;
            ORANGE
        };
        draw_block(x, y, color)
    }

    {
        // Draw Fruit
        let Position((x, y)) = state.fruit;
        draw_block(x, y, GREEN);
    }

    root_ui().label(None, "Use arrow keys to control the snake.");
    if state.dead {
        root_ui().label(None, "YOU DIED. R I P");
        root_ui().label(None, "Press 'Esc' to restart.");
    } else {
        // TODO(zac): Allocating every frame!
        root_ui().label(None, &format!("length of {}", state.parts.len()));
    }
}

fn input_to_direction(current: Direction, key: KeyCode, mapping: Direction) -> Option<Direction> {
    if is_key_pressed(key) && !is_opposite_of(mapping, current) {
        Some(mapping)
    } else {
        None
    }
}

#[macroquad::main("Snake")]
async fn main() {
    let mut state = reset_state();

    {
        let label_style = root_ui().style_builder().text_color(WHITE).build();
        let skin = macroquad::ui::Skin {
            label_style,
            ..root_ui().default_skin()
        };
        root_ui().push_skin(&skin);
    }

    set_camera(&Camera2D::from_display_rect(Rect {
        x: 0.0,
        y: 0.0,
        w: 11.0 * PART_WIDTH,
        h: 11.0 * PART_HEIGHT,
    }));

    loop {
        update(&mut state);
        render(&state);
        next_frame().await
    }
}
