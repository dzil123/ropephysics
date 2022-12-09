use pixels::{Pixels, SurfaceTexture};
use winit::{
    dpi::LogicalSize,
    event::{Event, VirtualKeyCode},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use winit_input_helper::WinitInputHelper;

const WIDTH: u32 = 70;
const HEIGHT: u32 = 50;
const SCALE: f64 = 10.0;

fn process_segment(head: &(i32, i32), tail: &mut (i32, i32)) {
    let dt = (head.0 - tail.0, head.1 - tail.1);

    let (hx, hy) = *head;
    let cx = head.0 - dt.0.signum();
    let cy = head.1 - dt.1.signum();

    *tail = match (dt.0.abs(), dt.1.abs()) {
        (0, _) | (1, 2) => (hx, cy),
        (_, 0) | (2, 1) => (cx, hy),
        // (a, b) if a == b => (cx, cy),
        // _ => unreachable!(),
        _ => (cx, cy),
    };
}

fn process_dir(rope: &mut [(i32, i32)], dir: (i32, i32)) {
    let head = &mut rope[0];
    *head = (head.0 + dir.0, head.1 + dir.1);

    for idx in 0..rope.len() - 1 {
        let head = rope[idx];
        let tail = &mut rope[idx + 1];
        process_segment(&head, tail);
    }
}

fn process_head_pos(rope: &mut [(i32, i32)], new_head: (i32, i32)) {
    let head = &mut rope[0];
    *head = new_head;

    for idx in 0..rope.len() - 1 {
        let head = rope[idx];
        let tail = &mut rope[idx + 1];
        process_segment(&head, tail);
    }
}

struct App {
    rope: Vec<(i32, i32)>,
}

impl App {
    fn new() -> Self {
        let rope_len = 20;
        let center = (WIDTH as i32 / 2, HEIGHT as i32 / 2);
        Self {
            rope: vec![center; rope_len],
        }
    }

    fn map_pt(&self, pt: (i32, i32)) -> (i32, i32) {
        (pt.0, HEIGHT as i32 - pt.1)
    }

    fn draw(&mut self, screen: &mut [u8]) {
        let bg = [10, 10, 10, 0xff];
        let fg = [200, 200, 200, 0xff];

        // let max = screen.len() / 4;
        // for (i, pix) in screen.chunks_exact_mut(4).enumerate() {
        //     let v = (i as f32 / max as f32);
        //     let v = (v * 255.0) as u8;
        //     let col = [v, v, v, 0xff];
        //     pix.copy_from_slice(&col);
        // }
        // return;

        for pix in screen.chunks_exact_mut(4) {
            pix.copy_from_slice(&bg);
        }
        for &pt in self.rope.iter() {
            let pt = self.map_pt(pt);
            let pt = (pt.0 as u32, pt.1 as u32);
            let idx = (WIDTH * pt.1 + pt.0) as usize;
            let idx = idx * 4;

            // let v = screen[idx] + 10;
            // let fg = [v, v, v, 0xff];
            screen[idx..idx + 4].copy_from_slice(&fg);
        }
    }

    fn update(&mut self, pos: (i32, i32), screen: &mut [u8]) {
        let pos = self.map_pt(pos);
        let head = self.rope[0];
        let delta = (pos.0 - head.0, pos.1 - head.1);
        println!("{:?}", (head, pos, delta));
        // self.rope[0] = pos;

        // for new_head in line_drawing::WalkGrid::new(self.rope[0], pos) {
        // for new_head in line_drawing::Bresenham::new(self.rope[0], pos) {
        {
            let new_head = pos;
            process_head_pos(&mut self.rope, new_head);
            self.draw(screen);
        }

        // process_dir(&mut self.rope, delta);
    }
}

fn main() {
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();

    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        let scaled_size = LogicalSize::new(WIDTH as f64 * SCALE, HEIGHT as f64 * SCALE);
        WindowBuilder::new()
            .with_title("Sim")
            .with_inner_size(scaled_size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH, HEIGHT, surface_texture).unwrap()
    };

    let mut app = App::new();

    event_loop.run(move |event, _, control_flow| {
        if let Event::RedrawRequested(_) = event {
            // app.draw(pixels.get_frame_mut());
            pixels.render().unwrap();
        }

        if input.update(&event) {
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            let (mouse_cell, _mouse_prev_cell) = input
                .mouse()
                .map(|(mx, my)| {
                    let (dx, dy) = input.mouse_diff();
                    let prev_x = mx - dx;
                    let prev_y = my - dy;

                    let (mx_i, my_i) = pixels
                        .window_pos_to_pixel((mx, my))
                        .unwrap_or_else(|pos| pixels.clamp_pixel_pos(pos));

                    let (px_i, py_i) = pixels
                        .window_pos_to_pixel((prev_x, prev_y))
                        .unwrap_or_else(|pos| pixels.clamp_pixel_pos(pos));

                    ((mx_i as i32, my_i as i32), (px_i as i32, py_i as i32))
                })
                .unwrap_or_default();

            if input.mouse_pressed(0) || input.mouse_held(0) {
                // let delta = (
                //     mouse_cell.0 - mouse_prev_cell.0,
                //     mouse_cell.1 - mouse_prev_cell.1,
                // );
                app.update(mouse_cell, pixels.get_frame_mut());
                // println!("{:?}", (mouse_cell, mouse_prev_cell));
            }
        }

        if let Some(size) = input.window_resized() {
            pixels.resize_surface(size.width, size.height);
        }
        window.request_redraw();
    });
}
