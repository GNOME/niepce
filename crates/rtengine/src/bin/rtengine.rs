use rtengine::ffi::{
    decrease_ref, init_, initial_image_load, options_load, partial_profile_apply_to,
    proc_params_new, process_image, processing_job_create, profile_store_load_dynamic_profile,
    save_as_jpeg,
};

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

    init_();
    options_load();

    let mut error = 0_i32;
    cxx::let_cxx_string!(fname = &filename);
    let mut image = initial_image_load(&fname, true, &mut error);
    if image.is_null() {
        println!("Error, couldn't load image: {error}");
        return;
    }

    let mut proc_params = proc_params_new();
    let raw_params = unsafe { profile_store_load_dynamic_profile(image.pin_mut().get_meta_data()) };
    partial_profile_apply_to(&raw_params, proc_params.pin_mut(), false);

    let job = processing_job_create(image.pin_mut(), proc_params.as_ref().unwrap(), false);
    let imagefloat = unsafe { process_image(job.into_raw(), &mut error, false) };

    if imagefloat.is_null() {
        println!("Error, couldn't render image: {error}");
        return;
    }

    cxx::let_cxx_string!(fname = "foo.jpg");
    save_as_jpeg(&imagefloat, &fname, 100, 3);
    unsafe { decrease_ref(image.into_raw()) };
}
