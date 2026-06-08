use alfred_workflow_rs::Item;

pub(crate) fn first_item() -> Item {
    Item::with_arg("First", "first-arg").set_uid("uid-1")
}

pub(crate) fn second_item() -> Item {
    Item::with_arg("Second", "second-arg").set_uid("uid-2")
}

pub(crate) fn third_item() -> Item {
    Item::with_arg("Third", "third-arg").set_uid("uid-3")
}
