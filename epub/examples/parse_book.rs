use epub::Book;
use std::path::Path;

fn main() {
    let subscriber = tracing_subscriber::FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    let path = Path::new("testfiles/epubs/pride_and_prejudice.epub");
    let mut b = Book::new(path).unwrap();
    dbg!(b.items().next());
    dbg!(b.element("id20"));
    dbg!(b.next_item("id7"));
    let c = b.content("id11").unwrap();
    dbg!(c.content());
}
