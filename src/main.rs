use std::str::Bytes;

struct Node {
    tag: String,
    attributes: Vec<Attribute>,
    parent: usize,
    children: Vec<usize>,
}

struct Attribute {
    name: String,
    value: Option<String>,
}

enum State {
    EntityContent,
    EntityOpenBegin,
    EntityAttributes,
    EntityAttributeValue,
    EntitySelfClosing,
    TagOpenOrClose,
    EntityCloseBegin,
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

fn tokenize_document(document: &str) -> Vec<String> {
    let mut tokens: Vec<String> = Vec::new();
    let mut bytes = document.bytes();

    loop {
        let token = match consume_token(&mut bytes) {
            Some(t) => t,
            None => break,
        };
        tokens.push(token)
    }

    tokens
}

fn parse_attributes(attributes_str: &String) -> Vec<Attribute> {
    println!("{}", attributes_str);
    let mut attributes: Vec<Attribute> = Vec::new();
    for token in attributes_str.split_ascii_whitespace() {
        match token.split_once('=') {
            Some((name, value)) => {
                println!("name: {}, value: {}", name, value);
                attributes.push(Attribute { name: name.to_owned(), value: Some(value.to_owned()) });
            }
            None => {
                println!("name: {}", token);
                attributes.push(Attribute { name: token.to_owned(), value: None });
            }
        }
    }
    attributes
}

fn parse_tokens(tokens: &Vec<String>) -> Vec<Node> {
    let mut it = tokens.iter();
    let mut state = State::EntityContent;
    let root = Node { tag: "#document".to_owned(), attributes: vec![], parent: 0, children: Vec::new() };
    let mut nodes: Vec<Node> = vec![root];
    let mut head: usize = 0;
    let mut current_tag = String::new();
    let mut current_attributes = String::new();
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
                                State::EntityAttributes => {
                                    state = State::EntityAttributeValue;
                                }

                                State::EntityAttributeValue => {
                                    state = State::EntityAttributes;
                                }

                                _ => {}
                            }
                        }

                        '<' => {
                            match state {
                                State::EntityContent => {
                                    state = State::TagOpenOrClose;
                                },

                                // TokenizerState::EntityOpenEnd => {
                                //     // Push the current node onto the stack
                                //     stack.push(Node { tag: token_parsed.to_owned(), attributes: vec![], children: vec![] });
                                //     token_parsed = String::new();
                                // }

                                _ => {}
                            }
                        }

                        '>' => {
                            match state {
                                State::EntityOpenBegin | State::EntityAttributes => {
                                    println!("Pushed entity {}", current_tag);
                                    let idx = nodes.len();
                                    let attributes = parse_attributes(&current_attributes);
                                    nodes.push(Node { tag: current_tag.to_owned(), attributes, parent: head, children: vec![] });
                                    nodes[head].children.push(idx);
                                    head = idx;
                                    current_tag = String::new();
                                    current_attributes = String::new();
                                    state = State::EntityContent;
                                },

                                State::EntitySelfClosing => {
                                    println!("Pushed entity {}", current_tag);
                                    let idx = nodes.len();
                                    let attributes = parse_attributes(&current_attributes);
                                    nodes.push(Node { tag: current_tag.to_owned(), attributes, parent: head, children: vec![] });
                                    nodes[head].children.push(idx);
                                    current_tag = String::new();
                                    current_attributes = String::new();
                                    state = State::EntityContent;
                                },

                                State::EntityCloseBegin => {
                                    state = State::EntityContent;
                                    head = nodes[head].parent;
                                }

                                _ => {}
                            }
                        }

                        '/' => {
                            match state {
                                State::EntityAttributeValue => {
                                    current_attributes.push(c);
                                }

                                State::TagOpenOrClose => {
                                    state = State::EntityCloseBegin;
                                }

                                State::EntityAttributes | State::EntityOpenBegin => {
                                    state = State::EntitySelfClosing;
                                }

                                _ => {}
                            }
                        }

                        _ => {
                            match state {
                                State::EntityOpenBegin => {
                                    if token_boundary {
                                        state = State::EntityAttributes;
                                        current_attributes.push(c);
                                    } else {
                                        current_tag.push(c);
                                    }
                                }

                                State::EntityAttributes | State::EntityAttributeValue => {
                                    if token_boundary {
                                        current_attributes.push(' ');
                                    }
                                    current_attributes.push(c);
                                }

                                State::TagOpenOrClose => {
                                    state = State::EntityOpenBegin;
                                    current_tag.push(c);
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

fn main() {
    let s = include_str!("books.xml");
    let tokens = tokenize_document(s);
    let document = parse_tokens(&tokens);
    print_node(0, &document, 0);
}

fn print_node_header(node: &Node, depth: usize) {
    print!("{}{}", " ".repeat(depth * 2), node.tag);
    for Attribute { name, value } in node.attributes.iter() {
        // match value {
        //     Some(v) => {
        //         println!("val: {}", v);
        //     }
        //     None => {}
        // }
        print!(" [{}={}]", name, match value { Some(v) => v, None => ""});
    }
    println!("");
}

fn print_node(idx: usize, tree: &Vec<Node>, depth: usize) {
    let node = &tree[idx];
    // println!("{}{} [{}]", " ".repeat(depth * 2), node.tag, match node.attributes.len() > 0 { true => node.attributes[0].to_string(), false => "".to_string() });
    print_node_header(node, depth);
    for child in node.children.iter() {
        print_node(*child, tree, depth + 1);
    }
}
