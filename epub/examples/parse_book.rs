use std::path::Path;

use epub::Book;

fn main() {
    let path = Path::new("testfiles/epubs/pride_and_prejudice.epub");
    let b = Book::new(path).unwrap();
    dbg!(b.items().next());
    dbg!(b.next_item("id7"));
}
