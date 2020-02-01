use pyrite;
use pyrite::resources::PackagedProvider;

fn main() {
    pyrite::start(PackagedProvider::new());
}
