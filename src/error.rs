use std::error::Error;

pub fn io_err_to_box_err(e: std::io::Error) -> Box<dyn Error + Send> {
    Box::new(e)
}
