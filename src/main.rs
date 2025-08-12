#[forbid(unsafe_code)]
mod screen;
mod switch;
mod xrandr;

fn main() {
    println!("{:?}", xrandr::query_xrandr());
}
