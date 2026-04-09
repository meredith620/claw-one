use std::collections::HashMap;

#[test]
fn test_string_reverse() {
    let s = "hello";
    let reversed: String = s.chars().rev().collect();
    assert_eq!(reversed, "olleh");

    let empty: String = "".chars().rev().collect();
    assert_eq!(empty, "");

    let palindrome = "racecar";
    let rev: String = palindrome.chars().rev().collect();
    assert_eq!(rev, palindrome);
}

#[test]
fn test_vec_sum() {
    let nums = vec![1, 2, 3, 4, 5];
    let sum: i32 = nums.iter().sum();
    assert_eq!(sum, 15);

    let empty: Vec<i32> = vec![];
    let zero: i32 = empty.iter().sum();
    assert_eq!(zero, 0);

    let negatives = vec![-1, -2, 3];
    let result: i32 = negatives.iter().sum();
    assert_eq!(result, 0);
}

#[test]
fn test_hashmap_insert() {
    let mut map: HashMap<&str, i32> = HashMap::new();
    map.insert("a", 1);
    map.insert("b", 2);
    map.insert("c", 3);

    assert_eq!(map.len(), 3);
    assert_eq!(map.get("a"), Some(&1));
    assert_eq!(map.get("b"), Some(&2));
    assert!(!map.contains_key("z"));

    // Overwrite an existing key
    map.insert("a", 99);
    assert_eq!(map.get("a"), Some(&99));
    assert_eq!(map.len(), 3); // length unchanged after overwrite
}

#[test]
fn test_option_unwrap() {
    let some_val: Option<i32> = Some(42);
    assert_eq!(some_val.unwrap(), 42);
    assert_eq!(some_val.unwrap_or(0), 42);

    let none_val: Option<i32> = None;
    assert_eq!(none_val.unwrap_or(0), 0);
    assert_eq!(none_val.unwrap_or_default(), 0);

    // map on Option
    let doubled = some_val.map(|x| x * 2);
    assert_eq!(doubled, Some(84));

    let mapped_none = none_val.map(|x| x * 2);
    assert_eq!(mapped_none, None);
}

#[test]
fn test_result_ok() {
    let ok: Result<i32, &str> = Ok(10);
    assert!(ok.is_ok());
    assert_eq!(ok.unwrap(), 10);
    assert_eq!(ok.unwrap_or(0), 10);

    let err: Result<i32, &str> = Err("something went wrong");
    assert!(err.is_err());
    assert_eq!(err.unwrap_or(0), 0);
    assert_eq!(err.unwrap_or_default(), 0);

    // map on Result
    let doubled = ok.map(|x| x * 2);
    assert_eq!(doubled, Ok(20));

    // Converting to Option
    assert_eq!(ok.ok(), Some(10));
    assert_eq!(err.ok(), None);
}
