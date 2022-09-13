use pg::run;
use pg::config::Config;

fn main() {
    match Config::new(std::env::args().skip(1).collect::<Vec<String>>()) {
        Ok(cfg) => {
            pollster::block_on(run(cfg));
        }
        Err(err) => {
            eprintln!("{err}");
        }
    }
}
 