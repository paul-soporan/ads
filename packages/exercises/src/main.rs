use ads::BinarySearchTree;

fn bst() {
    println!("--- Binary Search Tree Demo ---\n");
    let mut bst = BinarySearchTree::<i32>::new();

    let initial_values = [50, 30, 70, 20, 40, 60, 80];
    println!("Inserting values: {:?}", initial_values);
    for &val in &initial_values {
        bst.insert(val);
    }

    if let Some(min_node) = bst.min() {
        println!("\nTree Minimum: {}", *min_node.value());
    }
    if let Some(max_node) = bst.max() {
        println!("Tree Maximum: {}", *max_node.value());
    }

    let target = 40;
    println!("\nExploring around the value '{}':", target);

    if let Some(pred) = bst.predecessor_of_value(&target) {
        println!("  Predecessor: {}", *pred.value());
    } else {
        println!("  Predecessor: None");
    }

    if let Some(succ) = bst.successor_of_value(&target) {
        println!("  Successor: {}", *succ.value());
    } else {
        println!("  Successor: None");
    }

    let to_delete = 30;
    println!("\nDeleting node '{}'...", to_delete);
    if let Some(deleted) = bst.delete_value(&to_delete) {
        println!("Successfully deleted: {}", deleted);
    }

    let check_val = 20;
    println!("\nChecking successor of '{}' after deletion:", check_val);
    if let Some(succ) = bst.successor_of_value(&check_val) {
        println!("  New Successor: {}", *succ.value());
    }

    println!("\nFinal tree state contains 50? {}", bst.contains(&50));
    println!("Final tree state contains 30? {}", bst.contains(&30));
}

fn main() {
    bst();
}
