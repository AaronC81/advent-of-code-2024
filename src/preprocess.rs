pub fn preprocess(input: &str) -> String {
    input.split("\n")
        .filter(|line| !line.trim().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}
