use pg::run;

fn main() {
    pollster::block_on(run());
}
 