use rtengine::RtEngine;

fn main() {
    let mut args = std::env::args();
    if args.len() < 2 {
        println!("Filename is needed");
        std::process::exit(1);
    }
    if args.len() > 2 {
        println!("Ignoring extra arguments");
    }
    args.next();
    let filename = args.next().expect("Expect an argument");

    let engine = RtEngine::new();
    if engine.set_file(filename, true /* is_raw */).is_err() {
        std::process::exit(3);
    }

    match engine.process() {
        Err(error) => {
            println!("Error, couldn't render image: {error}");
            std::process::exit(2);
        }
        Ok(image) => {
            image.save_png("foo.png").expect("Couldn't save image");
        }
    }
}
