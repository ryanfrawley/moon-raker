mod document;
// mod xpath;

fn main() {
    let s = include_str!("test.txt");
    let doc = document::build_document_tree(s);
    document::print_node(0, &doc);
}

