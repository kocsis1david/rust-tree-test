use node::Node;

#[allow(unused)]
mod node;

fn main() {
    let root = Node::new("root");
    let a = Node::new("a");
    root.add_child_last(&a);
    let b = Node::new("b");
    root.add_child_last(&b);
    let c = Node::new("c");
    root.add_child_last(&c);

    for child in root.children() {
        println!("{}", **child);
    }

    b.detach();

    for child in root.children() {
        println!("{}", **child);
    }
}
