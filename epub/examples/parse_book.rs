use epub::Book;
use std::path::Path;

fn main() {
    let subscriber = tracing_subscriber::FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    let path = Path::new("testfiles/epubs/pride_and_prejudice.epub");
    let mut b = Book::new(path).unwrap();
    println!("-- print all items in order");
    for i in b.items() {
        println!("{}", i.id());
    }

    println!("-- get first item");
    let n = b.items().next().unwrap();
    println!("first: {}", n.id());

    println!("-- get item by id");
    let n = b.element("item5").unwrap();
    println!("item5: {}", n.id());

    println!("-- get following item by (current) id");
    let n = b.next_item("item5").unwrap();
    println!("item5: {}", n.id());

    let c = b.content("pg-header").unwrap();
    let c = c.content();
    for i in 0..5 {
        dbg!(&c[i]);
    }
}
