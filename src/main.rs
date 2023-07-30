mod document;
// mod xpath;

fn main() {
    let s = include_str!("books.xml");
    let doc = document::build_document_tree(s);
    document::print_node(0, &doc);
}

