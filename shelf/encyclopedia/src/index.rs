use puzzle::Puzzle;
use std::collections::HashMap;

#[test]
fn test_search() {
    // TODO:

    let mut document = HashMap::new();

    document.insert("title", "The Old Man and the Sea");
    document.insert(
        "body",
        "He was an old man who fished alone in a skiff in the Gulf Stream and \
he had gone eighty-four days now without taking a fish.",
    );

    let mut value = bincode::serialize(&document).unwrap();

    let mut puzzle = Puzzle::empty();

    let key = puzzle.wrap(key);

    let context = context(&puzzle).unwrap();

    write(&puzzle, &context).unwrap();

    let got = read(&context, "sea whale").unwrap();

    dbg!(got);
}
