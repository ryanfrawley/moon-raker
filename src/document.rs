use std::str::Bytes;

pub struct Node {
    tag: String,
    attributes: Vec<Attribute>,
    parent: usize,
    children: Vec<usize>,
    value: Option<String>,
}

pub struct Attribute {
    name: String,
    value: Option<String>,
}

enum State {
    TagContent,
    TagOpenBegin,
    TagAttributes,
    TagAttributeValue,
    TagSelfClosing,
    TagOpenOrCloseBegin,
    TagCloseBegin,
}

fn consume_token(data: &mut Bytes) -> Option<String> {
    let mut token_started = false;
    let mut token: String = String::new();
    loop {
        match data.next() {
            Some(v) => {
                match v {
                    v if v.is_ascii_whitespace() => {
                        if token_started {
                            break;
                        }
                    }

                    _ => {
                        token_started = true;
                        token.push(v as char)
                    }
                }
            }

            None => break,
        }
    }

    match token_started {
        true => Some(token),
        false => None,
    }
}


fn parse_attributes(attributes_str: &String) -> Vec<Attribute> {
    let mut attributes: Vec<Attribute> = Vec::new();
    for token in attributes_str.split_ascii_whitespace() {
        match token.split_once('=') {
            Some((name, value)) => {
                attributes.push(Attribute { name: name.to_owned(), value: Some(value.to_owned()) });
            }
            None => {
                attributes.push(Attribute { name: token.to_owned(), value: None });
            }
        }
    }
    attributes
}

fn parse_tokens(tokens: &Vec<String>) -> Vec<Node> {
    let mut it = tokens.iter();
    let mut state = State::TagContent;
    let root = Node { tag: "#document".to_owned(), attributes: vec![], parent: 0, children: Vec::new(), value: None };
    let mut nodes: Vec<Node> = vec![root];
    let mut head: usize = 0;
    let mut current_tag = String::new();
    let mut current_attributes = String::new();
    let mut current_text = String::new();
    loop {
        match it.next() {
            Some(token) => {
                for (idx, c) in token.chars().enumerate() {
                    let token_boundary = idx == 0;

                    // println!("c: {}, state: {}", c, match state {
                    //     TokenizerState::EntityCloseBegin => "close begin",
                    //     TokenizerState::TagOpenOrClose => "tag open or close",
                    //     TokenizerState::EntityOpenBegin => "open begin",
                    //     TokenizerState::EntityContent => "content",
                    //     TokenizerState::EntityCloseEnd => "close end",
                    //     TokenizerState::EntityAttributes => "attributes",
                    //     _ => "invalid"
                    // });

                    match c {
                        '"' => {
                            match state {
                                State::TagAttributes => {
                                    state = State::TagAttributeValue;
                                }

                                State::TagAttributeValue => {
                                    state = State::TagAttributes;
                                }

                                _ => {}
                            }
                        }

                        '<' => {
                            match state {
                                State::TagContent => {
                                    state = State::TagOpenOrCloseBegin;
                                    if current_text.len() > 0 {
                                        let idx = nodes.len();
                                        nodes.push(Node { tag: "#text".to_string(), attributes: vec![], parent: head, children: vec![], value: Some(current_text.to_owned()) });
                                        nodes[head].children.push(idx);
                                        current_text = String::new();
                                    }
                                },
                                _ => {}
                            }
                        }

                        '>' => {
                            if current_tag.starts_with(['!', '?']) {
                                state = State::TagSelfClosing
                            }
                            match state {
                                State::TagOpenBegin | State::TagAttributes | State::TagSelfClosing => {
                                    // println!("Pushed entity {}", current_tag);
                                    let idx = nodes.len();
                                    let attributes = parse_attributes(&current_attributes);
                                    nodes.push(Node { tag: current_tag.to_owned(), attributes, parent: head, children: vec![], value: None });
                                    nodes[head].children.push(idx);
                                    head = match state {
                                        State::TagSelfClosing => head,
                                        State::TagOpenBegin | State::TagAttributes => idx,
                                        _ => panic!("Invalid state"),
                                    };
                                    current_tag = String::new();
                                    current_attributes = String::new();
                                    state = State::TagContent;
                                },

                                State::TagCloseBegin => {
                                    state = State::TagContent;
                                    head = nodes[head].parent;
                                }

                                _ => {}
                            }
                        }

                        '/' => {
                            match state {
                                State::TagAttributeValue => {
                                    current_attributes.push(c);
                                }

                                State::TagOpenOrCloseBegin => {
                                    state = State::TagCloseBegin;
                                }

                                State::TagAttributes | State::TagOpenBegin => {
                                    state = State::TagSelfClosing;
                                }

                                _ => {}
                            }
                        }

                        _ => {
                            match state {
                                State::TagOpenBegin => {
                                    if token_boundary {
                                        state = State::TagAttributes;
                                        current_attributes.push(c);
                                    } else {
                                        current_tag.push(c);
                                    }
                                }

                                State::TagAttributes | State::TagAttributeValue => {
                                    if token_boundary {
                                        current_attributes.push(' ');
                                    }
                                    current_attributes.push(c);
                                }

                                State::TagOpenOrCloseBegin => {
                                    state = State::TagOpenBegin;
                                    current_tag.push(c);
                                }

                                State::TagContent => {
                                    if token_boundary {
                                        current_text.push(' ');
                                    }
                                    current_text.push(c);
                                }

                                _ => {}
                            }
                        }
                    }
                }
            }

            None => break
        }
    }

    nodes

}

pub fn build_document_tree(document: &str) -> Vec<Node> {
    let mut tokens: Vec<String> = Vec::new();
    let mut bytes = document.bytes();
    loop {
        let token = match consume_token(&mut bytes) {
            Some(t) => t,
            None => break,
        };
        tokens.push(token)
    }

    parse_tokens(&tokens)
}

fn print_node_header(node: &Node, depth: usize) {
    print!("{}{}", " ".repeat(depth * 2), node.tag);
    for Attribute { name, value } in node.attributes.iter() {
        print!(" [{}={}]", name, match value { Some(v) => v, None => ""});
    }
    println!("");
    match &node.value {
        Some(v) => println!("{}{}", " ".repeat((depth + 1) * 2), v),
        None => {},
    }
}


fn print_node_with_depth(idx: usize, tree: &Vec<Node>, depth: usize) {
    let node = &tree[idx];
    print_node_header(node, depth);
    for child in node.children.iter() {
        print_node_with_depth(*child, tree, depth + 1);
    }
}

pub fn print_node(idx: usize, tree: &Vec<Node>) {
    print_node_with_depth(idx, tree, 0);
}
