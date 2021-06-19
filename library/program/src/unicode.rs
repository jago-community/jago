use unicode_segmentation::UnicodeSegmentation;

#[test]
fn test_to_upper_camel_case() {
    let cases = vec![("test", "Test"), ("a-test", "ATest")];

    for (input, want) in cases {
        let got = to_upper_camel_case(input);
        assert_eq!(&got, want);
    }
}

pub fn to_upper_camel_case(input: &str) -> String {
    let words = input.unicode_words();

    let output = words.fold(String::new(), |mut output, word| {
        output.push_str(&upper_first(word));
        output
    });

    output
}

fn upper_first(input: &str) -> String {
    let mut parts = input.graphemes(true);

    match parts.next() {
        None => String::new(),
        Some(first) => {
            first
                .chars()
                .map(|c| c.to_uppercase())
                .flat_map(|i| i)
                .collect::<String>()
                + parts.as_str()
        }
    }
}
