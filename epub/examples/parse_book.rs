use std::path::Path;

use epub::Book;

fn main() {
    let path = Path::new("testfiles/epubs/pride_and_prejudice.epub");
    let mut b = Book::new(path).unwrap();
    dbg!(b.items().next());
    dbg!(b.element("id20"));
    dbg!(b.next_item("id7"));
    let c = b.content("id20").unwrap();
    dbg!(c.content());
}
