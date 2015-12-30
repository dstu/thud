extern crate gtk;
extern crate cairo;
extern crate thud;

use ::thud::gtk_ui;
use std::fs::File;
use std::io::prelude::*;
use std::iter;

fn main() {
    let display = gtk_ui::DisplayProperties::new();

    let image_buffer = cairo::ImageSurface::create(
        cairo::Format::Rgb30,
        (display.margin_left + display.margin_right
            + display.cell_dimension * 15.0
            + display.border_width * 16.0) as i32,
        (display.margin_top + display.margin_bottom
            + display.cell_dimension * 15.0
         + display.border_width * 16.0) as i32);
    {
        let mut render_context = cairo::Context::new(&image_buffer);
        display.draw_board_decorations(&mut render_context);
    }

    // let mut data: Vec<u8> = iter::repeat(0u8).take(image_buffer.len()).collect();
    // image_buffer.get_data(&mut data);

    // let mut out = File::create("out.png").unwrap();
    // match out.write(&data) {
    //     Result::Ok(written) if written != image_buffer.len() =>
    //         panic!("Wrote only {} of {} bytes", written, image_buffer.len()),
    //     Result::Ok(written) => println!("Wrote {} bytes", written),
    //     _ => (),
    // }
    
    
    // match image_buffer.write_to_png("out.png") {
    //     cairo::Status::Success => (),
    //     cairo::Status::NoMemory => panic!("Cairo ran out of memory"),
    //     cairo::Status::SurfaceTypeMismatch =>
    //         panic!("Surface lacks pixel content"),
    //     cairo::Status::WriteError =>
    //         panic!("Cairo write error"),
    //     _ => panic!("Unknown Cairo error"),
    // };
}
